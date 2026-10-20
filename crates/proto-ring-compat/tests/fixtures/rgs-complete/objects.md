---
governed_objects:
  model_version: 1
  interfaces:
    governance:
      objects:
        object-a:
          responsibilities:
            - evidence-required
            - object-governance
          relations:
            - relation: depends_on
              responsibility: object-governance
              target:
                interface: governance
                object: object-b
        object-b:
          responsibilities:
            - object-governance
          relations: []
---
# Objects
