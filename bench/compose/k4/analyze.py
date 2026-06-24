#!/usr/bin/env python3
# [OPUS-4.8] sq-3pb7f. Compute the K4 accuracy A/B: per-arm accuracy, per-kind breakdown,
# paired delta (HIDDEN - RAW), and an exact paired sign test (no scipy). Pure stdlib.
import json, glob, os, math, sys
from collections import defaultdict

K4 = os.path.dirname(os.path.abspath(__file__))

def load_scores():
    s = {}
    for f in sorted(glob.glob(f"{K4}/scores_*.json")):
        for rec in json.load(open(f)):
            s[rec['file'].replace('.txt', '')] = float(rec['score'])
    return s

def binom_two_sided_p(k, n, p=0.5):
    # exact two-sided binomial p-value for k successes in n trials under p=0.5
    if n == 0:
        return 1.0
    def pmf(i):
        return math.comb(n, i) * (p ** i) * ((1 - p) ** (n - i))
    obs = pmf(k)
    # two-sided: sum of all outcomes at most as likely as observed
    return min(1.0, sum(pmf(i) for i in range(n + 1) if pmf(i) <= obs + 1e-12))

def main():
    keymap = json.load(open(f"{K4}/judge_keymap.json"))   # jid -> {id, arm, kind}
    scores = load_scores()                                # jid -> score
    # (id, arm) -> score
    by_qa = {}
    by_arm = defaultdict(list)
    by_kind_arm = defaultdict(lambda: defaultdict(list))
    missing = []
    for jid, meta in keymap.items():
        if jid not in scores:
            missing.append(jid)
            continue
        sc = scores[jid]
        by_qa[(meta['id'], meta['arm'])] = sc
        by_arm[meta['arm']].append(sc)
        by_kind_arm[meta['kind']][meta['arm']].append(sc)
    if missing:
        print(f"MISSING {len(missing)} scores: {missing[:10]}", file=sys.stderr)

    # Overall accuracy per arm
    def mean(xs):
        return sum(xs) / len(xs) if xs else float('nan')
    raw_acc = mean(by_arm['raw'])
    hid_acc = mean(by_arm['hidden'])

    # Paired per-question deltas (hidden - raw)
    ids = sorted({k[0] for k in by_qa})
    deltas = []
    pairs = []
    for qid in ids:
        if (qid, 'raw') in by_qa and (qid, 'hidden') in by_qa:
            r = by_qa[(qid, 'raw')]; h = by_qa[(qid, 'hidden')]
            deltas.append(h - r)
            pairs.append((qid, r, h, h - r))
    n_pairs = len(deltas)
    wins = sum(1 for d in deltas if d > 0)     # hidden better
    losses = sum(1 for d in deltas if d < 0)   # raw better
    ties = sum(1 for d in deltas if d == 0)
    mean_delta = mean(deltas)
    # Exact sign test over non-tie pairs
    n_eff = wins + losses
    k = max(wins, losses)
    p_sign = binom_two_sided_p(k, n_eff) if n_eff > 0 else 1.0

    out = {
        'n_questions': len(ids),
        'n_pairs': n_pairs,
        'raw_accuracy': round(raw_acc, 4),
        'hidden_accuracy': round(hid_acc, 4),
        'mean_delta_hidden_minus_raw': round(mean_delta, 4),
        'paired_hidden_wins': wins,
        'paired_raw_wins': losses,
        'paired_ties': ties,
        'sign_test_p_two_sided': round(p_sign, 4),
        'per_kind': {},
    }
    for kind in sorted(by_kind_arm):
        r = by_kind_arm[kind]['raw']; h = by_kind_arm[kind]['hidden']
        out['per_kind'][kind] = {
            'n': len(r),
            'raw_acc': round(mean(r), 3),
            'hidden_acc': round(mean(h), 3),
            'delta': round(mean(h) - mean(r), 3),
        }

    json.dump(out, open(f"{K4}/k4_result.json", 'w'), indent=2)
    json.dump(pairs, open(f"{K4}/k4_pairs.json", 'w'), indent=1)
    print(json.dumps(out, indent=2))

if __name__ == '__main__':
    main()
