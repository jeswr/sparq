#!/usr/bin/env python3
"""
Validator for literature-seeds.toml registry.

Checks:
- Required keys present in each seed (id, topic, attribute, query, max_records, rationale)
- Attribute values are in enum: performance | correctness | genai-self-improvement | feature-usefulness
- Topic values are in enum: LLM x knowledge-representation, neurosymbolic AI,
  query optimization / database systems, engine correctness + logic-bug testing,
  agentic memory + self-improvement, RDF / SPARQL systems, zk-adjacent
- max_records is positive integer
- id values are unique
- Optional self-test mode with injected bad entries
"""

import sys
import os
import re

# Try standard library toml if available (Python 3.11+), otherwise manual parsing
try:
    import tomllib
except ImportError:
    # Fallback: minimal TOML parser for [[seeds]] format only
    class TOMLError(Exception):
        pass

    class tomllib:
        @staticmethod
        def loads(content):
            """Minimal TOML parser for [[seeds]] arrays of tables."""
            result = {"seeds": []}
            current_seed = None

            for line_num, line in enumerate(content.split('\n'), 1):
                line = line.strip()

                # Skip comments and empty lines
                if not line or line.startswith('#'):
                    continue

                # New seed entry
                if line == '[[seeds]]':
                    if current_seed is not None:
                        result["seeds"].append(current_seed)
                    current_seed = {}
                    continue

                # Parse key-value pairs
                if current_seed is not None and '=' in line:
                    key, value = line.split('=', 1)
                    key = key.strip()
                    value = value.strip()

                    # Remove quotes from string values
                    if (value.startswith('"') and value.endswith('"')) or \
                       (value.startswith("'") and value.endswith("'")):
                        value = value[1:-1]
                    # Parse integers
                    elif value.isdigit():
                        value = int(value)

                    current_seed[key] = value

            # Don't forget the last seed
            if current_seed is not None:
                result["seeds"].append(current_seed)

            return result


REQUIRED_ATTRIBUTES = {
    "performance",
    "correctness",
    "genai-self-improvement",
    "feature-usefulness",
}

REQUIRED_TOPICS = {
    "LLM x knowledge-representation",
    "neurosymbolic AI",
    "query optimization / database systems",
    "engine correctness + logic-bug testing",
    "agentic memory + self-improvement",
    "RDF / SPARQL systems",
    "zk-adjacent",
}

REQUIRED_SEED_KEYS = {
    "id",
    "topic",
    "attribute",
    "query",
    "max_records",
    "rationale",
}


def validate_seeds(seeds, strict=True):
    """
    Validate seeds list.

    Args:
        seeds: List of seed dicts from TOML
        strict: If True, treat all violations as errors; if False, report warnings

    Returns:
        (is_valid, errors, warnings)
    """
    errors = []
    warnings = []
    seen_ids = set()

    if not isinstance(seeds, list):
        errors.append("Seeds must be a list")
        return False, errors, warnings

    if len(seeds) == 0:
        errors.append("At least one seed is required")
        return False, errors, warnings

    topics_seen = set()
    attributes_seen = set()

    for idx, seed in enumerate(seeds, 1):
        if not isinstance(seed, dict):
            errors.append(f"Seed {idx}: not a dict")
            continue

        # Check required keys
        missing_keys = REQUIRED_SEED_KEYS - set(seed.keys())
        if missing_keys:
            errors.append(
                f"Seed {idx} (id={seed.get('id', 'UNKNOWN')}): "
                f"missing required keys: {', '.join(sorted(missing_keys))}"
            )
            continue

        seed_id = seed.get("id")
        topic = seed.get("topic")
        attribute = seed.get("attribute")
        query = seed.get("query")
        max_records = seed.get("max_records")
        rationale = seed.get("rationale")

        # Validate id uniqueness
        if seed_id in seen_ids:
            errors.append(f"Seed {idx}: duplicate id '{seed_id}'")
        else:
            seen_ids.add(seed_id)

        # Validate id format (alphanumeric + hyphens, no leading/trailing hyphens)
        if not re.match(r'^[a-z0-9]+(-[a-z0-9]+)*$', str(seed_id)):
            errors.append(
                f"Seed {idx} (id={seed_id}): id must be lowercase alphanumeric with hyphens"
            )

        # Validate topic
        if topic not in REQUIRED_TOPICS:
            errors.append(
                f"Seed {idx} (id={seed_id}): unknown topic '{topic}'. "
                f"Must be one of: {', '.join(sorted(REQUIRED_TOPICS))}"
            )
        else:
            topics_seen.add(topic)

        # Validate attribute
        if attribute not in REQUIRED_ATTRIBUTES:
            errors.append(
                f"Seed {idx} (id={seed_id}): unknown attribute '{attribute}'. "
                f"Must be one of: {', '.join(sorted(REQUIRED_ATTRIBUTES))}"
            )
        else:
            attributes_seen.add(attribute)

        # Validate query (non-empty string)
        if not isinstance(query, str) or len(query.strip()) == 0:
            errors.append(f"Seed {idx} (id={seed_id}): query must be a non-empty string")

        # Validate rationale (non-empty string)
        if not isinstance(rationale, str) or len(rationale.strip()) == 0:
            errors.append(f"Seed {idx} (id={seed_id}): rationale must be a non-empty string")

        # Validate max_records (positive integer)
        if not isinstance(max_records, int) or max_records <= 0:
            errors.append(
                f"Seed {idx} (id={seed_id}): max_records must be a positive integer, "
                f"got {max_records}"
            )

    # Check coverage of required topics
    missing_topics = REQUIRED_TOPICS - topics_seen
    if missing_topics:
        errors.append(
            f"Missing required topics (registry must cover all six): "
            f"{', '.join(sorted(missing_topics))}"
        )

    is_valid = len(errors) == 0
    return is_valid, errors, warnings


def validate_file(filepath):
    """Validate a TOML file."""
    if not os.path.exists(filepath):
        print(f"ERROR: File not found: {filepath}")
        return False

    try:
        with open(filepath, 'r') as f:
            content = f.read()

        data = tomllib.loads(content)
        seeds = data.get('seeds', [])

        is_valid, errors, warnings = validate_seeds(seeds)

        if warnings:
            for warn in warnings:
                print(f"WARN: {warn}", file=sys.stderr)

        if errors:
            for err in errors:
                print(f"ERROR: {err}", file=sys.stderr)

        if is_valid:
            seed_count = len(seeds)
            topics = set(s.get('topic') for s in seeds)
            print(f"OK: {seed_count} seeds, {len(topics)} topics, all required topics covered")
            return True
        else:
            return False

    except Exception as e:
        print(f"ERROR: Failed to parse TOML: {e}", file=sys.stderr)
        return False


def self_test():
    """Run self-test with injected bad entries."""
    print("Running self-test with fixtures...", file=sys.stderr)

    # Good fixture
    good_seeds = [
        {
            "id": "test-seed",
            "topic": "LLM x knowledge-representation",
            "attribute": "genai-self-improvement",
            "query": "test query",
            "max_records": 10,
            "rationale": "test seed for validation",
        },
        {
            "id": "test-seed-2",
            "topic": "neurosymbolic AI",
            "attribute": "correctness",
            "query": "another test",
            "max_records": 20,
            "rationale": "another test seed",
        },
        {
            "id": "test-seed-3",
            "topic": "query optimization / database systems",
            "attribute": "performance",
            "query": "query test",
            "max_records": 30,
            "rationale": "third seed",
        },
        {
            "id": "test-seed-4",
            "topic": "engine correctness + logic-bug testing",
            "attribute": "correctness",
            "query": "test",
            "max_records": 15,
            "rationale": "fourth seed",
        },
        {
            "id": "test-seed-5",
            "topic": "agentic memory + self-improvement",
            "attribute": "genai-self-improvement",
            "query": "test",
            "max_records": 25,
            "rationale": "fifth seed",
        },
        {
            "id": "test-seed-6",
            "topic": "RDF / SPARQL systems",
            "attribute": "feature-usefulness",
            "query": "test",
            "max_records": 35,
            "rationale": "sixth seed",
        },
        {
            "id": "test-seed-7",
            "topic": "zk-adjacent",
            "attribute": "correctness",
            "query": "zero knowledge proof test",
            "max_records": 20,
            "rationale": "seventh seed for zk-adjacent coverage",
        },
    ]

    is_valid, errors, warnings = validate_seeds(good_seeds)
    if not is_valid:
        print(f"SELF-TEST FAILED: good fixture rejected", file=sys.stderr)
        for err in errors:
            print(f"  {err}", file=sys.stderr)
        return False

    print("  ✓ Good fixture passed", file=sys.stderr)

    # Bad fixture: duplicate id
    bad_dup_id = good_seeds[:2] + [{**good_seeds[0], "topic": "neurosymbolic AI"}]
    is_valid, errors, _ = validate_seeds(bad_dup_id)
    if is_valid:
        print(f"SELF-TEST FAILED: duplicate id not caught", file=sys.stderr)
        return False
    if not any("duplicate id" in e for e in errors):
        print(f"SELF-TEST FAILED: expected duplicate id error", file=sys.stderr)
        return False
    print("  ✓ Duplicate id rejection passed", file=sys.stderr)

    # Bad fixture: invalid attribute
    bad_attr = [good_seeds[0].copy()]
    bad_attr[0]["attribute"] = "invalid-attribute"
    is_valid, errors, _ = validate_seeds(bad_attr)
    if is_valid:
        print(f"SELF-TEST FAILED: invalid attribute not caught", file=sys.stderr)
        return False
    print("  ✓ Invalid attribute rejection passed", file=sys.stderr)

    # Bad fixture: non-positive max_records
    bad_records = [good_seeds[0].copy()]
    bad_records[0]["max_records"] = 0
    is_valid, errors, _ = validate_seeds(bad_records)
    if is_valid:
        print(f"SELF-TEST FAILED: non-positive max_records not caught", file=sys.stderr)
        return False
    print("  ✓ Non-positive max_records rejection passed", file=sys.stderr)

    # Bad fixture: missing required key
    bad_missing = [good_seeds[0].copy()]
    del bad_missing[0]["rationale"]
    is_valid, errors, _ = validate_seeds(bad_missing)
    if is_valid:
        print(f"SELF-TEST FAILED: missing key not caught", file=sys.stderr)
        return False
    print("  ✓ Missing key rejection passed", file=sys.stderr)

    # Bad fixture: missing required topic
    bad_topics = good_seeds[:6]  # Missing "zk-adjacent"
    is_valid, errors, _ = validate_seeds(bad_topics)
    if is_valid:
        print(f"SELF-TEST FAILED: missing required topic not caught", file=sys.stderr)
        return False
    if not any("Missing required topics" in e for e in errors):
        print(f"SELF-TEST FAILED: expected missing topic error", file=sys.stderr)
        return False
    print("  ✓ Missing required topic rejection passed", file=sys.stderr)

    print("SELF-TEST PASSED: all fixtures validated correctly", file=sys.stderr)
    return True


def main():
    """Main entry point."""
    if len(sys.argv) > 1 and sys.argv[1] == "--self-test":
        success = self_test()
        return 0 if success else 1

    # Find literature-seeds.toml relative to this script
    script_dir = os.path.dirname(os.path.abspath(__file__))
    seeds_file = os.path.join(script_dir, "literature-seeds.toml")

    if not os.path.exists(seeds_file):
        print(f"ERROR: Could not find {seeds_file}", file=sys.stderr)
        return 1

    success = validate_file(seeds_file)
    return 0 if success else 1


if __name__ == "__main__":
    sys.exit(main())
