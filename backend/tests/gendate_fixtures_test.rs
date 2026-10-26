use genealogy_backend::models::gendate::{CalendarType, GenDate, GenDateMod, PartialDate};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GenDateFixtureCase {
    input: String,
    locale: String,
    expected: GenDate,
    sort_key: Option<i32>,
    is_valid: bool,
}

#[test]
fn test_shared_gendate_fixtures_roundtrip_sortkey_and_parse() {
    let raw = include_str!("../../fixtures/gendate_cases.json");
    let cases: Vec<GenDateFixtureCase> =
        serde_json::from_str(raw).expect("fixtures/gendate_cases.json must deserialize cleanly");

    assert!(
        cases.len() >= 15,
        "Expected at least 15 canonical GenDate fixture cases"
    );

    for case in &cases {
        assert!(
            case.locale == "en" || case.locale == "ru",
            "Unexpected locale '{}' for input '{}'",
            case.locale,
            case.input
        );
        assert!(case.is_valid, "Case '{}' should be valid", case.input);

        // 1. Verify JSON serialization/deserialization round-trip of expected GenDate
        let serialized =
            serde_json::to_value(&case.expected).expect("GenDate must serialize to JSON value");
        let deserialized: GenDate =
            serde_json::from_value(serialized).expect("GenDate must deserialize from JSON value");
        assert_eq!(
            deserialized, case.expected,
            "Round-trip mismatch for '{}'",
            case.input
        );

        // 2. Verify compute_sort_key() matches fixture sortKey
        assert_eq!(
            case.expected.compute_sort_key(),
            case.sort_key,
            "Sort key mismatch for '{}'",
            case.input
        );

        // 3. Verify bilingual parser produces exact expected GenDate and sortKey
        let parsed = GenDate::parse(&case.input);
        assert_eq!(
            parsed, case.expected,
            "Parser output mismatch for input '{}'",
            case.input
        );
        assert_eq!(
            parsed.compute_sort_key(),
            case.sort_key,
            "Parsed sort key mismatch for input '{}'",
            case.input
        );
    }
}

#[test]
fn test_julian_month_and_year_rollover_sort_key() {
    // 25 Dec 1905 (Julian) + 13 days = 7 Jan 1906 (Gregorian) -> 19060107
    let dec_julian = GenDate {
        r#mod: GenDateMod::Exact,
        start: Some(PartialDate {
            y: 1905,
            m: Some(12),
            d: Some(25),
        }),
        end: None,
        cal: CalendarType::Julian,
        text: "25.12.1905 (ст. ст.)".to_string(),
    };
    assert_eq!(dec_julian.compute_sort_key(), Some(19_060_107));

    // 19th century Julian (+12 days): 20 Feb 1850 + 12 days = 4 Mar 1850 -> 18500304
    let nineteenth_c = GenDate {
        r#mod: GenDateMod::Exact,
        start: Some(PartialDate {
            y: 1850,
            m: Some(2),
            d: Some(20),
        }),
        end: None,
        cal: CalendarType::Julian,
        text: "20.02.1850 (ст. ст.)".to_string(),
    };
    assert_eq!(nineteenth_c.compute_sort_key(), Some(18_500_304));
}
