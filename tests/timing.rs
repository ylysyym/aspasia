use aspasia::{Moment, TimeDelta};

#[test]
fn moment_equality() {
    let zero = Moment::from(0);

    assert_eq!(zero, Moment::from(0));
    assert_eq!(0_i64, zero.into());
    assert_ne!(zero, Moment::from(1));
}

#[test]
fn moment_comparison() {
    let negative = Moment::from(-1);
    let zero = Moment::from(0);
    let positive = Moment::from(1);

    assert!(positive > negative);
    assert!(positive > zero);
    assert!(negative < zero);
    assert!(positive >= negative);
    assert!(positive >= zero);
    assert!(negative <= zero);
}

#[test]
fn edge_conversions() {
    let before = Moment::from(60 * 60 * 1000 - 1);
    let after: Moment = Moment::from(60 * 60 * 1000 + 1);

    assert_eq!(before.hours(), 0);
    assert_eq!(before.minutes(), 59);
    assert_eq!(after.hours(), 1);
    assert_eq!(after.minutes(), 0);
}

#[test]
fn delta_operations() {
    let mut delta = TimeDelta::from(10 * 1000);
    let other_delta = TimeDelta::from(20 * 1000);

    assert_eq!(delta, TimeDelta::from(10 * 1000));
    assert_eq!(delta + other_delta, TimeDelta::from(30 * 1000));
    assert_eq!(delta - other_delta, TimeDelta::from(-10 * 1000));
    assert_eq!(4 * delta, TimeDelta::from(40 * 1000));
    assert_eq!(delta / 10, TimeDelta::from(1000));

    delta += other_delta;

    assert_eq!(delta, TimeDelta::from(30 * 1000));

    delta -= other_delta;

    assert_eq!(delta, TimeDelta::from(10 * 1000));

    delta *= 2;

    assert_eq!(delta, TimeDelta::from(20 * 1000));

    delta /= 4;

    assert_eq!(delta, TimeDelta::from(5 * 1000));
}

#[test]
fn moment_delta_operations() {
    let mut time = Moment::from(0);
    let delta = TimeDelta::from(1000);

    assert_eq!(time + delta, Moment::from(1000));
    assert_eq!(time - delta, Moment::from(-1000));
    assert_eq!(delta + time, Moment::from(1000));

    time += delta;

    assert_eq!(time, Moment::from(1000));

    time -= delta;

    assert_eq!(time, Moment::from(0));
}
