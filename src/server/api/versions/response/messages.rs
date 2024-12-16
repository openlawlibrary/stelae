use chrono::NaiveDate;
use serde::Serialize;

use super::format_date;
use crate::server::api::versions::response::Version;

/// Messages for the versions endpoint.
#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Historical {
    /// Message for an outdated publication.
    pub publication: Option<String>,
    /// Message for an outdated version.
    pub version: Option<String>,
    /// Message for a comparison between two versions.
    pub comparison: Option<String>,
}

/// Returns historical messages for the versions endpoint.
/// The historical messages currently include:
/// - A message for an outdated publication.
/// - A message for an outdated version.
/// - A message for a comparison between two versions.
#[must_use]
pub fn historical(
    versions: &[Version],
    current_publication_name: &str,
    active_publication_name: &str,
    version_date: &Option<String>,
    compare_to_date: &Option<String>,
) -> Historical {
    let current_version: &str = versions
        .first()
        .map(|lmv| lmv.date.as_str())
        .unwrap_or_default();

    let publication = publication_message(
        active_publication_name,
        current_publication_name,
        current_version,
    );
    let version = version_date.as_ref().and_then(|found_version_date| {
        version_message(
            current_version,
            found_version_date,
            versions,
            compare_to_date.as_ref(),
        )
    });
    let comparison = compare_to_date.as_ref().and_then(|found_compare_to_date| {
        version_date.as_ref().map(|found_version_date| {
            comparison_message(
                found_compare_to_date,
                found_version_date,
                current_version,
                versions,
            )
        })
    });
    Historical {
        publication,
        version,
        comparison,
    }
}

/// Returns a historical message for an outdated publication.
fn publication_message(
    active_publication_name: &str,
    current_publication_name: &str,
    current_version: &str,
) -> Option<String> {
    if active_publication_name == current_publication_name {
        return None;
    }
    Some(publication_message_template(current_version))
}

/// Formats the response for an outdated publication.
fn publication_message_template(date: &str) -> String {
    format!(
        "You are viewing a historical publication that was last updated on {current_date} and is no longer being updated.",
        current_date = format_date(date)
    )
}

/// Returns a historical message for an outdated version.
/// Version is outdated if `version_date` is in the past.
fn version_message(
    current_version: &str,
    version_date: &str,
    versions: &[Version],
    compare_to_date: Option<&String>,
) -> Option<String> {
    let is_current_version = {
        let current_date =
            NaiveDate::parse_from_str(current_version, "%Y-%m-%d").unwrap_or_default();
        let Ok(parsed_version_date) = NaiveDate::parse_from_str(version_date, "%Y-%m-%d") else {
            return None;
        };
        current_date <= parsed_version_date
    };
    if compare_to_date.is_some() || is_current_version {
        return None;
    }
    let version_date_idx = versions
        .iter()
        .position(|ver| ver.date.as_str() == version_date);
    let (start_date, end_date) = version_date_idx.map_or_else(
        || {
            let end_date = versions
                .iter()
                .filter(|ver| ver.date.as_str() > version_date)
                .map(|ver| ver.date.as_str())
                .min();
            let found_idx = versions
                .iter()
                .position(|ver| ver.date.as_str() == end_date.unwrap_or_default())
                .unwrap_or_default();
            let start_date = versions
                .get(found_idx + 1)
                .map_or_else(|| versions.last(), Some)
                .map(|ver| ver.date.as_str());
            (start_date.unwrap_or_default(), end_date.unwrap_or_default())
        },
        |idx| {
            let start_date = version_date;
            let end_date = versions
                .get(idx - 1)
                .map_or_else(|| versions.first(), Some)
                .map(|ver| ver.date.as_str())
                .unwrap_or_default();
            (start_date, end_date)
        },
    );
    Some(version_message_template(version_date, start_date, end_date))
}

/// Formats the response for an outdated version.
fn version_message_template(version_date: &str, start_date: &str, end_date: &str) -> String {
    format!(
        "You are viewing this document as it appeared on {version_date}. This version was valid between {start_date} and {end_date}.",
        version_date = format_date(version_date),
        start_date = format_date(start_date),
        end_date = format_date(end_date)
    )
}

/// Returns a historical message for a comparison between two versions.
fn comparison_message(
    compare_to_date: &str,
    version_date: &str,
    current_date: &str,
    versions: &[Version],
) -> String {
    let (compare_start_date, compare_end_date) = if version_date > compare_to_date {
        (compare_to_date, version_date)
    } else {
        (version_date, compare_to_date)
    };
    let start_idx = Version::find_index_or_closest(versions, compare_start_date);
    let end_idx = Version::find_index_or_closest(versions, compare_end_date);
    let num_of_changes = start_idx - end_idx;
    let start_date = format_date(compare_start_date);
    let end_date = if compare_end_date == current_date {
        None
    } else {
        Some(format_date(compare_end_date))
    };
    messages_between_template(num_of_changes, &start_date, end_date)
}

/// Formats and returns a message for the number of changes between two dates.
fn messages_between_template(
    num_of_changes: usize,
    start_date: &str,
    end_date: Option<String>,
) -> String {
    let changes = match num_of_changes {
        0 => "no updates".to_owned(),
        1 => "1 update".to_owned(),
        _ => format!("{num_of_changes} updates"),
    };

    end_date.map_or_else(
        || format!("There have been <strong>{changes}</strong> since {start_date}."),
        |found_end_date| {
            format!(
                "There have been <strong>{changes}</strong> between {start_date} and {found_end_date}."
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    use std::cmp::Reverse;
    use std::collections::BTreeMap;

    use super::super::Publication;

    fn publication_to_versions() -> BTreeMap<Reverse<String>, Publication> {
        let test_data = json!({
            "2023-12-30": {
                "active": false,
                "date": "2023-12-30",
                "display": "2023-12-30",
                "name": "2023-12-30",
                "versions": [
                    {"date": "2023-12-30", "display": "2023-12-30", "version": 0},
                    {"date": "2023-12-11", "display": "2023-12-11", "version": 0},
                    {"date": "2023-11-02", "display": "2023-11-02", "version": 0},
                    {"date": "2023-10-22", "display": "2023-10-22", "version": 0},
                    {"date": "2023-08-12", "display": "2023-08-12", "version": 0},
                    {"date": "2023-08-10", "display": "2023-08-10", "version": 0},
                    {"date": "2023-06-04", "display": "2023-06-04", "version": 0},
                    {"date": "2023-01-01", "display": "2023-01-01", "version": 0}
                ]
            },
            "2023-10-22": {
                "active": false,
                "date": "2023-10-22",
                "display": "2023-10-22",
                "name": "2023-10-22",
                "versions": [
                    {"date": "2023-10-22", "display": "2023-10-22", "version": 0},
                    {"date": "2023-08-12", "display": "2023-08-12", "version": 0},
                    {"date": "2023-08-10", "display": "2023-08-10", "version": 0},
                    {"date": "2023-06-04", "display": "2023-06-04", "version": 0},
                    {"date": "2023-01-01", "display": "2023-01-01", "version": 0}
                ]
            }
        });
        let map: BTreeMap<Reverse<String>, Publication> =
            serde_json::from_value(test_data).unwrap();
        map
    }

    fn current_publication_name() -> String {
        "2023-12-30".to_string()
    }

    #[test]
    fn test_historical_when_current_publication_expect_no_historical_messages() {
        let active_publication_name = "2023-12-30".to_string();
        let current_publication_name = current_publication_name();
        let publication_to_versions = publication_to_versions();
        let versions = &publication_to_versions
            .get(&Reverse(active_publication_name.clone()))
            .unwrap()
            .versions;
        let version_date: Option<String> = None;
        let compare_to_date: Option<String> = None;

        let cut = historical;

        let actual = cut(
            &versions,
            &current_publication_name,
            &active_publication_name,
            &version_date,
            &compare_to_date,
        );
        let expected = Historical {
            publication: None,
            version: None,
            comparison: None,
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_historical_when_outdated_publication_expect_publication_message_with_last_update() {
        let test_cases = vec![
            None,
            Some("2023-10-22".to_string()),
            Some("2024-06-06".to_string()),
        ];

        for version_date in test_cases {
            let active_publication_name = "2023-10-22".to_string();
            let current_publication_name = current_publication_name();
            let publication_to_versions = publication_to_versions();
            let versions = &publication_to_versions
                .get(&Reverse(active_publication_name.clone()))
                .unwrap()
                .versions;
            let compare_to_date: Option<String> = None;

            let cut = historical;

            let actual = cut(
                &versions,
                &current_publication_name,
                &active_publication_name,
                &version_date,
                &compare_to_date,
            );
            let expected = Historical {
                publication: Some(publication_message_template(&versions[0].date)),
                version: None,
                comparison: None,
            };

            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_historical_when_outdated_publication_and_outdated_date_expect_publication_and_version_message_with_last_update(
    ) {
        let test_cases = vec![
            ("2023-01-01", "2023-01-01", "2023-06-04"), // first date
            ("2023-08-12", "2023-08-12", "2023-10-22"), // middle date
            ("2023-02-02", "2023-01-01", "2023-06-04"), // non-existing date
        ];

        for (version_date, start_date, end_date) in test_cases {
            let active_publication_name = "2023-10-22".to_string();
            let current_publication_name = current_publication_name();
            let publication_to_versions = publication_to_versions();
            let versions = &publication_to_versions
                .get(&Reverse(active_publication_name.clone()))
                .unwrap()
                .versions;
            let compare_to_date: Option<String> = None;

            let cut = historical;

            let actual = cut(
                &versions,
                &current_publication_name,
                &active_publication_name,
                &Some(version_date.to_string()),
                &compare_to_date,
            );
            let expected = Historical {
                publication: Some(publication_message_template(&versions[0].date)),
                version: Some(version_message_template(version_date, start_date, end_date)),
                comparison: None,
            };

            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_historical_when_comparing_with_latest_date_expect_historical_message_with_comparison_date(
    ) {
        let test_cases = vec![
            ("2023-10-22", "no updates"),
            ("2023-08-12", "1 update"),
            ("2023-08-10", "2 updates"),
            ("2023-07-01", "3 updates"), // non-existing date
            ("2023-01-01", "4 updates"),
            ("2020-01-01", "5 updates"), // non-existing date before creation date
        ];

        for (version_date, changes) in test_cases {
            let active_publication_name = "2023-10-22".to_string();
            let current_publication_name = current_publication_name();
            let publication_to_versions = publication_to_versions();
            let versions = &publication_to_versions
                .get(&Reverse(active_publication_name.clone()))
                .unwrap()
                .versions;
            let compare_to_date = Some("2023-10-22".to_string());
            let start_date = version_date;

            let cut = historical;

            let actual = cut(
                &versions,
                &current_publication_name,
                &active_publication_name,
                &Some(version_date.to_string()),
                &compare_to_date,
            );

            let expected_comparison_message = messages_between_template(
                match changes {
                    "no updates" => 0,
                    "1 update" => 1,
                    "2 updates" => 2,
                    "3 updates" => 3,
                    "4 updates" => 4,
                    "5 updates" => 5,
                    _ => 0,
                },
                &format_date(start_date),
                None,
            );

            let expected = Historical {
                publication: Some(publication_message_template(&versions[0].date)),
                version: None,
                comparison: Some(expected_comparison_message),
            };

            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn test_historical_messages_when_comparing_with_non_latest_date_expect_historical_message() {
        let test_cases = vec![
            ("2023-12-11", "2023-12-11", "no updates"),
            ("2023-10-22", "2023-11-02", "1 update"),
            ("2023-10-22", "2023-12-11", "2 updates"),
            ("2023-07-01", "2023-12-11", "5 updates"), // non-existing start date
            ("2023-06-04", "2023-09-11", "2 updates"), // non-existing end date
            ("2023-07-01", "2023-09-11", "2 updates"), // non-existing start and end date
            ("2020-01-01", "2023-06-04", "2 updates"), // non-existing start date before creation date
            ("2020-01-01", "2020-06-04", "no updates"), // non-existing start and end date before creation date
            ("2020-01-01", "2024-06-04", "8 updates"),  // end date in the future
            ("2023-07-01", "2024-06-04", "6 updates"), // non-existing start date and end date in the future
        ];

        for (version_date, compare_to_date, changes) in test_cases {
            let active_publication_name = "2023-12-30".to_string();
            let current_publication_name = current_publication_name();
            let publication_to_versions = publication_to_versions();
            let versions = &publication_to_versions
                .get(&Reverse(active_publication_name.clone()))
                .unwrap()
                .versions;

            let cut = historical;

            let actual = cut(
                &versions,
                &current_publication_name,
                &active_publication_name,
                &Some(version_date.to_string()),
                &Some(compare_to_date.to_string()),
            );

            let expected_comparison_message = messages_between_template(
                match changes {
                    "no updates" => 0,
                    "1 update" => 1,
                    "2 updates" => 2,
                    "3 updates" => 3,
                    "4 updates" => 4,
                    "5 updates" => 5,
                    "6 updates" => 6,
                    "7 updates" => 7,
                    "8 updates" => 8,
                    _ => 0,
                },
                &format_date(version_date),
                Some(format_date(compare_to_date)),
            );

            let expected = Historical {
                publication: None,
                version: None,
                comparison: Some(expected_comparison_message),
            };

            assert_eq!(actual, expected);
        }
    }
}
