//! HTML notification banners injected into historical document responses.
//!
//! Two notifications are used by the `date` endpoint:
//! - [`outdated_doc`] – shown when the requested version is not the latest.
//! - [`outdated_pub`] – shown when the requested publication is not the latest.
//!
//! The remaining functions cover comparison-page banners and are
//! reserved for the future `compare` endpoint.

/// Renders the plain-text message for an outdated document notification.
#[must_use]
pub fn outdated_doc_message(date: &str, start_date: &str, end_date: &str) -> String {
    format!(
        "You are viewing this document as it appeared on {date}. \
        This version was valid between {start_date} and {end_date}."
    )
}

/// Renders the full HTML banner for an outdated document notification.
///
/// Used by the `date` endpoint when the requested `version_date` is not the
/// latest version of the document.
#[must_use]
pub fn outdated_doc(date: &str, start_date: &str, end_date: &str, current_doc_url: &str) -> String {
    format!(
        r#"
<div class="message message--info" role="region" tabindex="0">
    <div class="h__message">HISTORICAL DOCUMENT</div>
    <p>
        {}
        <br/>
        <br/>
        Click <a href="{current_doc_url}">here</a> to see the current version.
    </p>
</div>
"#,
        outdated_doc_message(date, start_date, end_date)
    )
}

/// Renders the plain-text message for an outdated publication notification.
#[must_use]
pub fn outdated_pub_message(date: &str) -> String {
    format!(
        "You are viewing a historical publication that was last updated on {date} \
        and is no longer being updated."
    )
}

/// Renders the full HTML banner for an outdated publication notification.
///
/// Used by the `date` endpoint when the requested publication is not the
/// latest publication for the stelae.
#[must_use]
pub fn outdated_pub(date: &str, current_doc_url: &str) -> String {
    format!(
        r#"
<div class="message message--info" role="region" tabindex="0">
    <div class="h__message">HISTORICAL PUBLICATION</div>
    <p>
        {}
        <br/>
        <br/>
        Click <a href="{current_doc_url}">here</a> to navigate to the current publication.
    </p>
</div>
"#,
        outdated_pub_message(date)
    )
}

/// Renders the plain-text message when there are no changes since `start_date`.
#[must_use]
pub fn no_changes_since_message(start_date: &str) -> String {
    format!("There have been <strong>no updates</strong> since {start_date}.")
}

/// Renders the full HTML banner when there are no changes since `start_date`.
#[must_use]
pub fn no_changes_since(start_date: &str) -> String {
    format!(
        r#"
<div class="message message--info" role="region" tabindex="0">
    <p>
        {}
    </p>
</div>
"#,
        no_changes_since_message(start_date)
    )
}

/// Renders the plain-text message when there is exactly one change since `start_date`.
#[must_use]
pub fn one_change_since_message(start_date: &str) -> String {
    format!("There has been <strong>1 update</strong> since {start_date}.")
}

/// Renders the full HTML banner when there is exactly one change since `start_date`.
#[must_use]
pub fn one_change_since(start_date: &str) -> String {
    format!(
        r#"
<div class="message message--info" role="region" tabindex="0">
    <p>
        {}
    </p>
</div>
"#,
        one_change_since_message(start_date)
    )
}

/// Renders the plain-text message when there are multiple changes since `start_date`.
#[must_use]
pub fn multiple_changes_since_message(num_of_changes: usize, start_date: &str) -> String {
    format!("There have been <strong>{num_of_changes} updates</strong> since {start_date}.")
}

/// Renders the full HTML banner when there are multiple changes since `start_date`.
#[must_use]
pub fn multiple_changes_since(num_of_changes: usize, start_date: &str) -> String {
    format!(
        r#"
<div class="message message--info" role="region" tabindex="0">
    <p>
        {}
    </p>
</div>
"#,
        multiple_changes_since_message(num_of_changes, start_date)
    )
}

/// Renders the plain-text message when there are no changes between two dates.
#[must_use]
pub fn no_changes_between_message(start_date: &str, end_date: &str) -> String {
    format!("There have been <strong>no updates</strong> between {start_date} and {end_date}.")
}

/// Renders the full HTML banner when there are no changes between two dates.
#[must_use]
pub fn no_changes_between(start_date: &str, end_date: &str) -> String {
    format!(
        r#"
<div class="message message--info" role="region" tabindex="0">
    <p>
        {}
    </p>
</div>
"#,
        no_changes_between_message(start_date, end_date)
    )
}

/// Renders the plain-text message when there is exactly one change between two dates.
#[must_use]
pub fn one_change_between_message(start_date: &str, end_date: &str) -> String {
    format!("There has been <strong>1 update</strong> between {start_date} and {end_date}.")
}

/// Renders the full HTML banner when there is exactly one change between two dates.
#[must_use]
pub fn one_change_between(start_date: &str, end_date: &str) -> String {
    format!(
        r#"
<div class="message message--info" role="region" tabindex="0">
    <p>
        {}
    </p>
</div>
"#,
        one_change_between_message(start_date, end_date)
    )
}

/// Renders the plain-text message when there are multiple changes between two dates.
#[must_use]
pub fn multiple_changes_between_message(
    num_of_changes: usize,
    start_date: &str,
    end_date: &str,
) -> String {
    format!(
        "There have been <strong>{num_of_changes} updates</strong> between {start_date} and {end_date}."
    )
}

/// Renders the full HTML banner when there are multiple changes between two dates.
#[must_use]
pub fn multiple_changes_between(num_of_changes: usize, start_date: &str, end_date: &str) -> String {
    format!(
        r#"
<div class="message message--info" role="region" tabindex="0">
    <p>
        {}
    </p>
</div>
"#,
        multiple_changes_between_message(num_of_changes, start_date, end_date)
    )
}
