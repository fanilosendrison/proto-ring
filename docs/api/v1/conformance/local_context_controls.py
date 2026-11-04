"""Candidate local-context validation only; no host authority or API execution."""


def validate_unique_realizations(document):
    """Run after shape validation and before map construction; preserve exact IDs."""
    seen = set()
    for realization in document['realizations']:
        environment_id = realization['environment_id']
        if environment_id in seen:
            raise ValueError('local_context_invalid: duplicate environment_id')
        seen.add(environment_id)
