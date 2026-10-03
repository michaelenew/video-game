#!/usr/bin/env python3
"""Numbers as differences in kind: the analysis behind
docs/design/exploration/0006_differences_in_kind.md.

Reads what `cargo run --release -p hunt --bin spread` wrote (default
target/spread) and prints the results in Markdown: the contract rate (M0),
separability at matched difficulty (M1), identifiability from one run (M2),
and the per-knob kind / degree / dead ranking with cliffs (M3).

Standard library only.

    python3 scripts/spread.py [--dir target/spread] [--pairs 5]
"""

import argparse
import collections
import csv
import glob
import math
import random
import statistics

# The fight contract's two yardsticks (crates/hunt/tests/fight.rs): the
# Champion's committed attack's startup and active frames, and its poke's whiff
# cost. Printed by `crates/lab`'s `consts` binary.
OPENING_NEEDED = 24
POKE_WHIFF = 18
LIMIT = 36_000

ap = argparse.ArgumentParser()
ap.add_argument("--dir", default="target/spread")
ap.add_argument("--pairs", type=int, default=5)
ap.add_argument("--draws", type=int, default=600)
args = ap.parse_args()
rng = random.Random(1)


def load(regime):
    rows = []
    for f in sorted(glob.glob(f"{args.dir}/{regime}.*.hunts.csv")):
        with open(f) as fh:
            # A shard still running can end in half a row: keep whole ones.
            rows += [r for r in csv.DictReader(fh) if None not in r.values() and r.get(f"landed0", "") != ""]
    genomes = collections.defaultdict(list)
    for f in sorted(glob.glob(f"{args.dir}/{regime}.*.genomes.csv")):
        with open(f) as fh:
            next(fh)
            for line in fh:
                # A knob's id can hold a comma ("hit,_forward"): the setting is
                # the first field and the last five are fixed, the id is between.
                if not line.endswith("\n"):
                    continue
                parts = line.rstrip("\n").split(",")
                sid, knob, tail = int(parts[0]), ",".join(parts[1:-5]), parts[-5:]
                if knob:
                    genomes[sid].append((knob, int(tail[1]), int(tail[2]), int(tail[3]), int(tail[4])))
                else:
                    genomes.setdefault(sid, [])
    return rows, genomes


def moves_of(row):
    return sum(1 for k in row if k.startswith("start"))


def features(r):
    n = moves_of(r)
    fought = max(int(r["fought"]), 1)
    frames = max(int(r["frames"]), 1)
    mins = fought / 3600
    starts = [int(r[f"start{k}"]) for k in range(n)]
    landed = [int(r[f"landed{k}"]) for k in range(n)]
    tot = sum(starts)
    f = {}
    for k in range(n):
        f[f"mix{k}"] = starts[k] / tot if tot else 0
        f[f"hit{k}"] = landed[k] / starts[k] if starts[k] else 0
    thr = [int(r[f"threat{i}"]) for i in range(5)]
    ts = sum(thr) or 1
    for i in range(5):
        f[f"thr{i}"] = thr[i] / ts
    g = lambda k: int(r[k])
    f["react"] = g("reactable") / g("committed") if g("committed") else 0
    f["open_pm"] = g("openings") / mins
    f["open_mean"] = g("open_frames") / g("openings") if g("openings") else 0
    f["idle"] = g("idle_frames") / fought
    f["dominant"] = max(starts) / tot if tot else 0
    ent = -sum((s / tot) * math.log2(s / tot) for s in starts if s) if tot else 0
    f["entropy"] = ent / math.log2(n)
    f["coverage"] = sum(1 for s in starts if s) / n
    f["moves_pm"] = tot / mins
    f["ride_share"] = g("ride_frames") / frames
    f["rides_pm"] = g("rides") / mins
    f["ride_mean"] = g("ride_frames") / g("rides") if g("rides") else 0
    f["bucked"] = g("thrown") / g("rides") if g("rides") else 0
    f["fled"] = g("fled") / g("rides") if g("rides") else 0
    f["topple_pm"] = g("topples") / mins
    f["ridge_pm"] = g("ridge_hits") / mins
    f["legs"] = g("legs_broken")
    f["foot_share"] = g("foot_damage") / g("dealt") if g("dealt") else 0
    f["swing_pm"] = g("swings") / mins
    f["connect"] = g("connected") / g("swings") if g("swings") else 0
    f["spread"] = float(r["spread"])
    f["range"] = float(r["commit_range"])
    f["aboard"] = g("aboard_commits") / g("beats") if g("beats") else 0
    if r["uses"]:
        for i, u in enumerate(r["uses"].split("|")):
            f[f"use{i}"] = int(u) / mins
    return f


def difficulty_parts(r):
    won = r["outcome"] == "won"
    return (1.0 if won else 0.0, float(r["health_left"]), float(r["frames"]) if won else float(LIMIT))


# ---------------------------------------------------------------- baseline
base_rows, _ = load("base")
if not base_rows:
    raise SystemExit("no baseline: run spread --regime base first")
CLASSES = sorted({r["class"] for r in base_rows})
SEEDS_BASE = sorted({int(r["seed"]) for r in base_rows})

base = collections.defaultdict(dict)  # class -> seed -> features
base_d = []
for r in base_rows:
    base[r["class"]][int(r["seed"])] = features(r)
    base_d.append(difficulty_parts(r))

KEYS = {}
MEAN = {}
SD = {}
for c in CLASSES:
    keys = sorted(set().union(*[set(v) for v in base[c].values()]))
    keep = []
    for k in keys:
        vals = [base[c][s].get(k, 0.0) for s in SEEDS_BASE]
        sd = statistics.pstdev(vals)
        if sd > 1e-9:
            keep.append(k)
            MEAN[(c, k)] = statistics.fmean(vals)
            SD[(c, k)] = sd
    KEYS[c] = keep
DIM = sum(len(KEYS[c]) for c in CLASSES)

D_MEAN = [statistics.fmean(x[i] for x in base_d) for i in range(3)]
D_SD = [statistics.pstdev([x[i] for x in base_d]) or 1.0 for i in range(3)]


def std_vec(c, f):
    return [(f.get(k, 0.0) - MEAN[(c, k)]) / SD[(c, k)] for k in KEYS[c]]


def signature(per_class):
    """per_class: class -> list of feature dicts (one per seed)."""
    out = []
    for c in CLASSES:
        vs = [std_vec(c, f) for f in per_class.get(c, [])]
        if not vs:
            out += [0.0] * len(KEYS[c])
            continue
        out += [sum(col) / len(vs) for col in zip(*vs)]
    return out


def diff_score(parts):
    m = [statistics.fmean(p[i] for p in parts) for i in range(3)]
    z = [(m[i] - D_MEAN[i]) / D_SD[i] for i in range(3)]
    return (-z[0] - z[1] + z[2]) / 3


def dist(a, b):
    return math.sqrt(sum((x - y) ** 2 for x, y in zip(a, b)))


CLIP = 3.0


def dist_c(a, b):
    """The same distance with every component's difference capped at three
    noise units, so one rare event cannot carry the whole of it."""
    return math.sqrt(sum(min((x - y) ** 2, CLIP * CLIP) for x, y in zip(a, b)))




def norm(a):
    return math.sqrt(sum(x * x for x in a))


# Noise: two disjoint draws of S seeds from the baseline.
S = 12
noise, noise_c, d_noise = [], [], []
for _ in range(args.draws):
    pick = rng.sample(SEEDS_BASE, 2 * S)
    a, b = pick[:S], pick[S:]
    sa = signature({c: [base[c][s] for s in a] for c in CLASSES})
    sb = signature({c: [base[c][s] for s in b] for c in CLASSES})
    noise.append(dist(sa, sb))
    noise_c.append(dist_c(sa, sb))
    by = {(r["class"], int(r["seed"])): difficulty_parts(r) for r in base_rows}
    da = diff_score([by[(c, s)] for c in CLASSES for s in a])
    d_noise.append(da)
noise.sort()
noise_c.sort()
N95 = noise[int(0.95 * len(noise))]
N95C = noise_c[int(0.95 * len(noise_c))]
NAMES = [f"{c}:{k}" for c in CLASSES for k in KEYS[c]]
N50 = noise[len(noise) // 2]
SD_D = statistics.pstdev(d_noise)


# ---------------------------------------------------------------- settings
def settings(regime):
    rows, genomes = load(regime)
    per = collections.defaultdict(lambda: collections.defaultdict(dict))
    dparts = collections.defaultdict(list)
    champ = collections.defaultdict(list)
    for r in rows:
        sid = int(r["setting"])
        per[sid][r["class"]][int(r["seed"])] = features(r)
        dparts[sid].append(difficulty_parts(r))
        if r["class"] == "Champion":
            champ[sid].append(r)
    out = {}
    for sid in per:
        if any(len(per[sid][c]) == 0 for c in CLASSES):
            continue
        out[sid] = {
            "sig": signature({c: list(per[sid][c].values()) for c in CLASSES}),
            "D": diff_score(dparts[sid]),
            "wins": {c: None for c in CLASSES},
            "per": per[sid],
            "genome": genomes.get(sid, []),
            "champ": champ[sid],
            "rows": None,
        }
    for r in rows:
        sid = int(r["setting"])
        if sid in out:
            w = out[sid]["wins"]
            w[r["class"]] = (w[r["class"]] or 0) + (r["outcome"] == "won")
    return out


def contract(hunts):
    """The clauses of crates/hunt/tests/fight.rs these Champion hunts break."""
    broken = []
    if not hunts:
        return ["no hunts"]
    n = moves_of(hunts[0])
    g = lambda r, k: int(r[k])
    r0 = hunts[0]
    ans, dmg = g(r0, "reactable_moves"), g(r0, "damaging_moves")
    if not (ans * 2 > dmg and ans < dmg):
        broken.append("reactable mix")
    ever = [sum(g(r, f"start{k}") for r in hunts) for k in range(n)]
    if any(e == 0 for e in ever):
        broken.append("whole move set")
    for r in hunts:
        st = [g(r, f"start{k}") for k in range(n)]
        if sum(st) and max(st) / sum(st) >= 0.55 or g(r, "longest_repeat") > 6:
            broken.append("no single move")
            break
    if any((g(r, "open_frames") / g(r, "openings") if g(r, "openings") else 0) <= OPENING_NEEDED for r in hunts):
        broken.append("openings punishable")
    for r in hunts:
        fought = max(g(r, "fought"), 1)
        mpm = sum(g(r, f"start{k}") for k in range(n)) * 3600 / fought
        if g(r, "idle_frames") / fought >= 0.35 or mpm <= 20:
            broken.append("never idle")
            break
    for r in hunts:
        thr = [g(r, f"threat{i}") for i in range(5)]
        ts = sum(thr) or 1
        t, w, b = thr[0] / ts, thr[3] / ts, (thr[1] + thr[2]) / ts
        if not (0.35 <= t < 0.62) or w <= 0.12 or b <= 0.2:
            broken.append("threat bands")
            break
    rides = sum(g(r, "rides") for r in hunts)
    thrown = sum(g(r, "thrown") + g(r, "fled") for r in hunts)
    ridge = sum(g(r, "ridge_hits") for r in hunts)
    if rides == 0 or ridge == 0 or thrown * 4 <= rides or any(g(r, "ride_frames") / max(g(r, "frames"), 1) >= 0.8 for r in hunts):
        broken.append("back reachable, not safe")
    rf = sum(g(r, "ride_frames") for r in hunts)
    rn = sum(max(g(r, "rides"), 1) for r in hunts)
    if rf / rn <= POKE_WHIFF:
        broken.append("ride long enough")
    if sum(g(r, "topples") for r in hunts) == 0:
        broken.append("poise breaks")
    if any(g(r, "unanswerable") > 0 for r in hunts):
        broken.append("unanswerable zero")
    if any(r["outcome"] == "unresolved" for r in hunts):
        broken.append("hunt concludes")
    won = sum(r["outcome"] == "won" for r in hunts)
    if won == 0 or won == len(hunts):
        broken.append("neither free nor hopeless")
    if any(float(r["spread"]) <= 4.0 for r in hunts):
        broken.append("uses the arena")
    return broken


print("# Numbers as differences in kind — results\n")
print(f"Baseline: {len(SEEDS_BASE)} seeds x {len(CLASSES)} classes. Signature: {DIM} components "
      f"({', '.join(f'{c} {len(KEYS[c])}' for c in CLASSES)}), each in units of one seed's standard deviation.\n")
print(f"Noise floor at {S} seeds: median distance {N50:.2f}, **N95 = {N95:.2f}** (clipped: {N95C:.2f}). "
      f"Difficulty noise at {S} seeds a class: sd(D) = {SD_D:.3f}.\n")

base_champ = [r for r in base_rows if r["class"] == "Champion"]
base_first = sorted(base_champ, key=lambda r: int(r["seed"]))
seeds12 = None

results = {}
for regime in ["mutate", "wild"]:
    st = settings(regime)
    if st:
        results[regime] = st

# Seeds the regimes used (the first setting's), for the baseline's own contract on them.
for st in results.values():
    any_set = next(iter(st.values()))
    seeds12 = sorted(any_set["per"]["Champion"].keys())
    break
if seeds12:
    by_seed = {int(r["seed"]): r for r in base_champ}
    base_contract = contract([by_seed[s] for s in seeds12 if s in by_seed])
else:
    base_contract = []

# ---------------------------------------------------------------- M0
print("## M0 · The contract rate\n")
print(f"The Ridgeback as tuned, on the same {len(seeds12 or [])} seeds, breaks: "
      f"{', '.join(base_contract) or 'nothing'}. A genome *breaks the contract* if it breaks any clause the tuned creature keeps on these seeds.\n")
print("| Regime | Genomes | Break the contract | Most broken clauses | Unresolved hunts |\n| --- | --- | --- | --- | --- |")
for regime, st in results.items():
    count = collections.Counter()
    bad = 0
    for sid, s in st.items():
        b = [x for x in contract(s["champ"]) if x not in base_contract]
        s["broken"] = b
        if b:
            bad += 1
        count.update(b)
    unres = sum(1 for s in st.values() for c in CLASSES for f in s["per"][c].values())
    rows, _ = load(regime)
    unresolved = sum(r["outcome"] == "unresolved" for r in rows) / max(len(rows), 1)
    top = ", ".join(f"{k} {v}" for k, v in count.most_common(5))
    print(f"| {regime} | {len(st)} | {bad} ({100 * bad / len(st):.0f}%) | {top} | {100 * unresolved:.1f}% |")
print()

# The genomes that keep the contract: a broken creature being different says
# nothing about whether a fair one can be.
for regime in list(results):
    kept = {sid: g for sid, g in results[regime].items() if not g.get("broken")}
    if len(kept) >= 20:
        results[f"{regime}, contract kept"] = kept

# ---------------------------------------------------------------- the difficulty direction
mut = results.get("mutate", {})
vhat = None
if mut:
    Ds = [s["D"] for s in mut.values()]
    Dm = statistics.fmean(Ds)
    Sm = [statistics.fmean(col) for col in zip(*[s["sig"] for s in mut.values()])]
    num = [0.0] * DIM
    den = 0.0
    for s in mut.values():
        dd = s["D"] - Dm
        den += dd * dd
        for i in range(DIM):
            num[i] += (s["sig"][i] - Sm[i]) * dd
    v = [x / den for x in num] if den else [0.0] * DIM
    vn = norm(v) or 1.0
    vhat = [x / vn for x in v]


def orth(delta):
    if not vhat:
        return norm(delta)
    p = sum(a * b for a, b in zip(delta, vhat))
    return norm([a - p * b for a, b in zip(delta, vhat)])


# ---------------------------------------------------------------- the control
tem = settings("temper")
TEMPER_ORTH = None
if tem and 0 in tem:
    print("## The control: tempers\n")
    print("Tempers change only how the creature thinks (glance, lead, decisiveness, strain), and were built to be the same fight, harder. If the measure calls them a different fight, it is too sensitive.\n")
    print("| Temper | Difficulty change (noise sd) | Distance / N95 | Orthogonal / N95 | Clipped distance / clipped N95 |\n| --- | --- | --- | --- | --- |")
    for t in sorted(tem):
        if t == 0:
            continue
        a, b = tem[t], tem[0]
        delta = [x - y for x, y in zip(a["sig"], b["sig"])]
        o = orth(delta)
        if t == max(tem):
            TEMPER_ORTH = o
        print(f"| {t} | {(a['D'] - b['D']) / SD_D:+.1f} | {dist(a['sig'], b['sig']) / N95:.2f} | {o / N95:.2f} | {dist_c(a['sig'], b['sig']) / N95C:.2f} |")
    print()

# ---------------------------------------------------------------- M1
print("## M1 · Separability at matched difficulty\n")
print(f"Pairs of genomes whose difficulty differs by less than half its own noise ({0.5 * SD_D:.3f}), and how far apart their fights are. Kind rate: the share of those pairs further apart than N95.\n")
print("*Clipped* uses the clipped distance against its own noise floor. *Beyond temper III* counts pairs whose shift orthogonal to difficulty is larger than the hardest temper's.\n")
print("| Regime | Spread of D (sd) | Matched pairs | Median distance / N95 | Kind rate | Orthogonal | Clipped | Beyond temper III |\n| --- | --- | --- | --- | --- | --- | --- | --- |")
best_pairs = {}
for regime, st in results.items():
    ids = list(st)
    pairs = []
    for i in range(len(ids)):
        for j in range(i + 1, len(ids)):
            a, b = st[ids[i]], st[ids[j]]
            if abs(a["D"] - b["D"]) < 0.5 * SD_D:
                d = dist(a["sig"], b["sig"])
                o = orth([x - y for x, y in zip(a["sig"], b["sig"])])
                pairs.append((d, o, ids[i], ids[j], dist_c(a["sig"], b["sig"])))
    if not pairs:
        print(f"| {regime} | | 0 | | | |")
        continue
    kind = sum(1 for p in pairs if p[0] > N95) / len(pairs)
    okind = sum(1 for p in pairs if p[1] > N95) / len(pairs)
    ckind = sum(1 for p in pairs if p[4] > N95C) / len(pairs)
    tkind = sum(1 for p in pairs if TEMPER_ORTH and p[1] > TEMPER_ORTH) / len(pairs)
    med = statistics.median(p[0] for p in pairs) / N95
    sdD = statistics.pstdev([s["D"] for s in st.values()])
    print(f"| {regime} | {sdD / SD_D:.1f} x noise | {len(pairs)} | {med:.2f} | **{100 * kind:.0f}%** | {100 * okind:.0f}% | {100 * ckind:.0f}% | {100 * tkind:.0f}% |")
    pairs.sort(reverse=True)
    best_pairs[regime] = pairs[: args.pairs]
print()

# ---------------------------------------------------------------- lenses
def mix_of(per):
    """The move mix, pooled over every class and seed: the share of the
    creature's moves each move was."""
    tot = collections.Counter()
    for c in CLASSES:
        for f in per[c].values():
            for k, v in f.items():
                if k.startswith("mix"):
                    tot[k] += v
    n = sum(tot.values()) or 1
    return {k: v / n for k, v in tot.items()}


def tvd(a, b):
    return 0.5 * sum(abs(a.get(k, 0) - b.get(k, 0)) for k in set(a) | set(b))


def winvec(per):
    return [statistics.fmean(1.0 if f.get("_won") else 0.0 for f in per[c].values()) for c in CLASSES]


# The baseline's own scatter in both lenses, at twelve seeds a class.
by = {(r["class"], int(r["seed"])): r for r in base_rows}
mix_noise, win_noise = [], []
for _ in range(args.draws):
    pick = rng.sample(SEEDS_BASE, 2 * S)
    halves = []
    for h in (pick[:S], pick[S:]):
        per = {c: {s: features(by[(c, s)]) for s in h} for c in CLASSES}
        wins = [sum(by[(c, s)]["outcome"] == "won" for s in h) / S for c in CLASSES]
        halves.append((mix_of(per), wins))
    mix_noise.append(tvd(halves[0][0], halves[1][0]))
    win_noise.append(sum(abs(x - y) for x, y in zip(halves[0][1], halves[1][1])))
mix_noise.sort()
win_noise.sort()
MIX95 = mix_noise[int(0.95 * len(mix_noise))]
WIN95 = win_noise[int(0.95 * len(win_noise))]


def wins_of(G):
    return [G["wins"][c] / max(len(G["per"][c]), 1) for c in CLASSES]


print("## Two lenses a player would recognise\n")
print(f"**Move mix**: the share of the creature's moves that differ between two fights (total variation, 0 to 1); noise at {S} seeds a class reaches {MIX95:.3f} (95th percentile). **Class wins**: how far the six classes' win rates move between them, summed (0 to 6); noise reaches {WIN95:.2f}.\n")
if tem and 0 in tem:
    print("| Control | Move mix differs | Class wins move |\n| --- | --- | --- |")
    for t in sorted(tem):
        if t:
            print(f"| temper {t} | {tvd(mix_of(tem[t]['per']), mix_of(tem[0]['per'])):.3f} | {sum(abs(x - y) for x, y in zip(wins_of(tem[t]), wins_of(tem[0]))):.2f} |")
    print()
print("| Regime | Matched pairs | Median move-mix difference | Beyond noise | Over a tenth of moves | Median class-win shift | Beyond noise |\n| --- | --- | --- | --- | --- | --- | --- |")
for regime, st in results.items():
    ids = list(st)
    mixes = {i: mix_of(st[i]["per"]) for i in ids}
    wv = {i: wins_of(st[i]) for i in ids}
    m, w = [], []
    for i in range(len(ids)):
        for j in range(i + 1, len(ids)):
            if abs(st[ids[i]]["D"] - st[ids[j]]["D"]) < 0.5 * SD_D:
                m.append(tvd(mixes[ids[i]], mixes[ids[j]]))
                w.append(sum(abs(x - y) for x, y in zip(wv[ids[i]], wv[ids[j]])))
    if m:
        print(f"| {regime} | {len(m)} | {statistics.median(m):.3f} | {100 * sum(x > MIX95 for x in m) / len(m):.0f}% | {100 * sum(x > 0.1 for x in m) / len(m):.0f}% | {statistics.median(w):.2f} | {100 * sum(x > WIN95 for x in w) / len(w):.0f}% |")
print()

# ---------------------------------------------------------------- M2
print("## M2 · Identifiability from one run\n")
print("Hold out one seed of a genome; name the genome from that single run (six hunts, one a class) by the nearest genome mean in its difficulty band. Chance is one over the band.\n")
print("| Regime | Band size | Accuracy | Chance | Times chance |\n| --- | --- | --- | --- | --- |")
for regime, st in results.items():
    ids = sorted(st, key=lambda i: st[i]["D"])
    band = 10
    bands = [ids[k : k + band] for k in range(0, len(ids), band)]
    bands = [b for b in bands if len(b) >= 8]
    seeds = sorted(next(iter(st.values()))["per"]["Champion"].keys())
    vec = {}
    for sid in ids:
        for s in seeds:
            vec[(sid, s)] = sum((std_vec(c, st[sid]["per"][c].get(s, {})) for c in CLASSES), [])
    right = total = 0
    for b in bands:
        for s in seeds:
            means = {}
            for sid in b:
                others = [vec[(sid, t)] for t in seeds if t != s]
                means[sid] = [sum(col) / len(others) for col in zip(*others)]
            for sid in b:
                guess = min(b, key=lambda o: dist(vec[(sid, s)], means[o]))
                right += guess == sid
                total += 1
    if total:
        acc = right / total
        chance = 1 / band
        print(f"| {regime} | {band} | {100 * acc:.0f}% | {100 * chance:.0f}% | **{acc / chance:.1f}x** |")
print()

# ---------------------------------------------------------------- the pairs, in words
for regime, pairs in best_pairs.items():
    st = results[regime]
    print(f"### The most separated matched pairs ({regime})\n")
    for d, o, a, b, dc in pairs:
        A, B = st[a], st[b]
        contrib = sorted(((x - y) ** 2, n, x - y) for x, y, n in zip(A["sig"], B["sig"], NAMES))[::-1][:4]
        share = sum(c[0] for c in contrib) / (d * d) if d else 0
        print(f"- **{a} and {b}**: distance {d:.1f} ({d / N95:.1f} x N95; clipped {dc / N95C:.1f} x), difficulty {A['D']:+.2f} and {B['D']:+.2f}. Four components carry {100 * share:.0f}% of it: " + ", ".join(f"{n} {v:+.1f}" for _, n, v in contrib) + ".")
        for name, G in (("  -", A), ("  -", B)):
            wins = " ".join(f"{c[:2]}{G['wins'][c]}" for c in CLASSES)
            mix = collections.defaultdict(float)
            for c in CLASSES:
                for f in G["per"][c].values():
                    for k, v in f.items():
                        if k.startswith("mix"):
                            mix[k] += v
            tot = sum(mix.values()) or 1
            topmix = ", ".join(f"move {k[3:]} {100 * v / tot:.0f}%" for k, v in sorted(mix.items(), key=lambda kv: -kv[1])[:3])
            big = sorted(G["genome"], key=lambda g: -abs(g[1] - g[2]) / max(g[4] - g[3], 1))[:5]
            knobs = ", ".join(f"{g[0]} {g[2]}->{g[1]}" for g in big)
            print(f"{name} wins {wins}; most thrown {topmix}; biggest changes: {knobs}")
    print()

# ---------------------------------------------------------------- M3
oat_rows, oat_genomes = load("oat")
if oat_rows and vhat:
    st = settings("oat")
    by_knob = collections.defaultdict(list)
    for sid, s in st.items():
        g = s["genome"]
        if g:
            by_knob[g[0][0]].append((g[0][1], g[0][2], g[0][3], g[0][4], sid))
        else:
            pass
    # The reference: the level at the tuned value, on the same seeds.
    refs = [s for s in st.values() if not s["genome"]]
    ranking = []
    for knob, levels in by_knob.items():
        baked = levels[0][1]
        ref = None
        for v, b, lo, hi, sid in levels:
            if v == b:
                ref = st[sid]
        if ref is None and refs:
            ref = refs[0]
        if ref is None:
            continue
        uniq = {}
        for v, b, lo, hi, sid in levels:
            uniq[v] = sid
        vals = sorted(uniq)
        kinds, degs = [], []
        for v in vals:
            s = st[uniq[v]]
            delta = [x - y for x, y in zip(s["sig"], ref["sig"])]
            kinds.append(orth(delta))
            degs.append(s["D"] - ref["D"])
        lo, hi = levels[0][2], levels[0][3]
        span = [v for v in vals]
        steps = [dist(st[uniq[span[i + 1]]]["sig"], st[uniq[span[i]]]["sig"]) for i in range(len(span) - 1)]
        cliff = None
        if steps and sum(steps) > 0:
            i = max(range(len(steps)), key=lambda k: steps[k])
            if steps[i] > 0.5 * sum(steps) and steps[i] > N95:
                cliff = (span[i], span[i + 1])
        kind = max(kinds) if kinds else 0
        degree = max(abs(x) for x in degs) / SD_D if degs else 0
        # Kind at matched difficulty: the biggest orthogonal shift among
        # levels within one noise-sd of the tuned difficulty.
        pure = max([k for k, dd in zip(kinds, degs) if abs(dd) < SD_D] or [0])
        label = "kind" if kind > N95 else ("degree" if degree > 2 else "dead")
        ranking.append((knob, label, kind / N95, degree, pure / N95, cliff, baked, lo, hi))
    print("## M3 · Which knobs carry kind\n")
    print(f"Every knob alone, at seven levels across its range and five across +-30% of tuned. *Kind* is the largest shift of the fight orthogonal to the difficulty direction, in N95 units; *degree* the largest change of difficulty in noise sds; *kind at matched D* the largest orthogonal shift among levels within one noise sd of the tuned difficulty. A knob is **kind** if its orthogonal shift passes N95, **degree** if not but its difficulty moves by more than two sds, **dead** otherwise. A **cliff** is one step between adjacent range levels carrying over half the sweep's whole movement.\n")
    tally = collections.Counter(r[1] for r in ranking)
    print(f"Of {len(ranking)} knobs: {tally['kind']} kind, {tally['degree']} degree, {tally['dead']} dead; {sum(1 for r in ranking if r[5])} with a cliff.\n")

    def group(k):
        head, _, field = k.partition(".")
        if "_·_" in head:
            return {"mind": "mind", "nerve": "nerve", "hide": "hide", "legs": "lame layer", "pose": "shake force"}.get(head.split("_·_")[1], head)
        if "." in field:
            field = field.split(".", 1)[1]
            if field in ("startup", "active", "recovery"):
                return "move: frame data"
            if field in ("ideal_range", "range_tolerance", "bearing_tolerance", "wants_the_target_at_(cos)"):
                return "move: when it is chosen"
            if field.startswith("appetite"):
                return "move: appetite"
            if field == "lockout_after_use":
                return "move: lockout"
            if field.startswith("hit") and field not in ("hitstun",):
                return "move: hit volume and travel"
            if field in ("advance_while_active",):
                return "move: hit volume and travel"
            return "move: what a hit does"
        if field == "size_(x)":
            return "size"
        if field in ("health", "breakable_part_health", "poise", "poise_regained_per_frame", "topple_length", "stumble_length", "flinch_length", "damage_that_flinches_it"):
            return "health, poise, stagger"
        return "movement"

    groups = collections.defaultdict(collections.Counter)
    for r in ranking:
        groups[group(r[0])][r[1]] += 1
    print("| Group | Knobs | Kind | Degree | Dead |\n| --- | --- | --- | --- | --- |")
    for gname, c in sorted(groups.items(), key=lambda kv: -sum(kv[1].values())):
        print(f"| {gname} | {sum(c.values())} | {c['kind']} | {c['degree']} | {c['dead']} |")
    print()
    print("Strongest kind knobs:\n")
    print("| Knob | Kind (x N95) | Kind at matched D (x N95) | Degree (sd) | Cliff |\n| --- | --- | --- | --- | --- |")
    for r in sorted(ranking, key=lambda r: -r[2])[:25]:
        print(f"| {r[0]} | {r[2]:.2f} | {r[4]:.2f} | {r[3]:.1f} | {'%d -> %d' % r[5] if r[5] else ''} |")
    print("\nStrongest degree knobs (by difficulty moved):\n")
    print("| Knob | Degree (sd) | Kind (x N95) | Label |\n| --- | --- | --- | --- |")
    for r in sorted(ranking, key=lambda r: -r[3])[:15]:
        print(f"| {r[0]} | {r[3]:.1f} | {r[2]:.2f} | {r[1]} |")
    dead = sorted(r[0] for r in ranking if r[1] == "dead")
    print(f"\nDead ({len(dead)}): {', '.join(dead)}\n")
    cl = [r for r in ranking if r[5]]
    if cl:
        print("Cliffs:\n")
        for r in cl:
            print(f"- {r[0]}: between {r[5][0]} and {r[5][1]} (tuned {r[6]}, range {r[7]} to {r[8]})")
        print()
