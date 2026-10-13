use super::*;

fn projection(id: &str) -> Projection {
    Projection {
        id: id.to_owned(),
        responsibility: "responsibility".to_owned(),
        canonical_source: "canonical".to_owned(),
        secondary_source: "secondary".to_owned(),
        mode: ProjectionMode::Reference,
        validation: "validation".to_owned(),
        generator_source: None,
        boundary_source: None,
        target: None,
        binding: None,
    }
}

fn compatible(left: Projection, right: Projection) -> bool {
    reject_active_ambiguity(&BTreeMap::from([
        (left.id.clone(), left),
        (right.id.clone(), right),
    ]))
    .is_ok()
}

#[test]
fn effective_ambiguity_key_includes_target_and_binding() {
    let left = projection("left");
    let mut distinct_target = projection("target");
    distinct_target.canonical_source = "other".to_owned();
    distinct_target.target = Some(GovernedObjectRef {
        interface_id: "interface".to_owned(),
        object_id: "object".to_owned(),
    });
    assert!(compatible(left, distinct_target));
    let mut binding_one = projection("binding-one");
    binding_one.binding = Some("one".to_owned());
    let mut binding_two = projection("binding-two");
    binding_two.binding = Some("two".to_owned());
    binding_two.mode = ProjectionMode::MechanicallyValidatedMaintained;
    assert!(compatible(binding_one, binding_two));
}

#[test]
fn historical_snapshot_is_excluded_from_active_collision() {
    let left = projection("left");
    let mut historical = projection("historical");
    historical.canonical_source = "other".to_owned();
    historical.mode = ProjectionMode::BoundedHistoricalSnapshot;
    historical.boundary_source = Some("boundary".to_owned());
    assert!(compatible(left, historical));
}
