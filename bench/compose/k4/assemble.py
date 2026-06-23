#!/usr/bin/env python3
# [OPUS-4.8] sq-3pb7f. Assemble per-arm answers into a graded record + blinded judge prompts.
import json, glob, os, sys, random

K4 = os.path.dirname(os.path.abspath(__file__))

def load_questions():
    return {q['id']: q for q in (json.loads(l) for l in open(f"{K4}/questions.jsonl"))}

def load_answers():
    ans = {}
    for f in sorted(glob.glob(f"{K4}/answers_*.json")):
        for rec in json.load(open(f)):
            ans[rec['file']] = rec['answer']
    return ans

def main():
    qs = load_questions()
    ans = load_answers()
    # Build (id, arm) -> answer
    rows = []
    missing = []
    for qid, q in qs.items():
        for arm in ('raw', 'hidden'):
            key = f"q{qid:02d}_{arm}.txt"
            if key not in ans:
                missing.append(key)
                continue
            rows.append({
                'id': qid, 'kind': q['kind'], 'arm': arm,
                'prompt': q['prompt'], 'gold': q['gold'],
                'answer': ans[key], 'n_bindings': q['n_bindings'],
            })
    if missing:
        print(f"MISSING {len(missing)} answers: {missing[:10]}", file=sys.stderr)
    json.dump(rows, open(f"{K4}/graded_pre.json", 'w'), indent=1)
    print(f"assembled {len(rows)} (question,arm) answers; {len(missing)} missing", file=sys.stderr)

    # Blinded judge prompts: shuffle so the judge can't infer arm from order. Each judge item
    # gets a stable jid; the arm is recorded ONLY in the key map (not shown to the judge).
    items = list(rows)
    random.seed(42)
    random.shuffle(items)
    os.makedirs(f"{K4}/judge", exist_ok=True)
    keymap = {}
    TEMPL = """You are grading whether a candidate answer to a question is CORRECT, given the gold (reference) answer. Judge the SEMANTIC correctness of the answer to the question — not exact wording. A paraphrase that conveys the same fact is CORRECT. An answer that is on the right topic but misses or contradicts the key fact is PARTIAL. A wrong, evasive, or "cannot determine" answer is INCORRECT.

QUESTION:
{prompt}

GOLD (reference) ANSWER:
{gold}

CANDIDATE ANSWER:
{answer}

Reply with a single JSON object: {{"score": <1.0 for correct | 0.5 for partial | 0.0 for incorrect>, "why": "<one short clause>"}}. JSON only."""
    for i, it in enumerate(items):
        jid = f"j{i:03d}"
        keymap[jid] = {'id': it['id'], 'arm': it['arm'], 'kind': it['kind']}
        open(f"{K4}/judge/{jid}.txt", 'w').write(
            TEMPL.format(prompt=it['prompt'], gold=it['gold'], answer=it['answer']))
    json.dump(keymap, open(f"{K4}/judge_keymap.json", 'w'), indent=0)
    print(f"wrote {len(items)} blinded judge prompts", file=sys.stderr)

if __name__ == '__main__':
    main()
