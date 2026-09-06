//! Turning an address into a place on the site, and back.
//!
//! An address holds both things a reader might want to send somebody: which
//! page, and which of the two themes it should be read in. `/lite/philosophy`
//! is a whole instruction.
//!
//! The theme comes first because it is the rarer thing to name and the easier
//! to strip: everything after it reads the same whichever theme is in front.
//! Nothing here touches the browser, so all of it can be checked.

/// The two ways the site can look.
///
/// `lite` is the honest name for the second one: no sky, no wider gamut, no
/// simulation running behind anything — Solarized and nothing else.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Alien,
    Lite,
}

impl Theme {
    pub fn slug(self) -> &'static str {
        match self {
            Theme::Alien => "alien",
            Theme::Lite => "lite",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "alien" => Some(Theme::Alien),
            "lite" => Some(Theme::Lite),
            _ => None,
        }
    }

    pub fn is_lite(self) -> bool {
        matches!(self, Theme::Lite)
    }

    pub fn of(lite: bool) -> Self {
        if lite {
            Theme::Lite
        } else {
            Theme::Alien
        }
    }
}

/// Pulls an address apart into the theme it names, if it names one, and the
/// page slug that follows.
///
/// A first segment that is not a theme is taken as the page, so `/philosophy`
/// still lands on the right page and only leaves the theme unsaid.
pub fn split(path: &str) -> (Option<Theme>, String) {
    let mut parts = path
        .split('/')
        .map(str::trim)
        .filter(|part| !part.is_empty());

    let Some(first) = parts.next() else {
        return (None, String::new());
    };

    match Theme::from_slug(first) {
        Some(theme) => (Some(theme), parts.next().unwrap_or_default().to_string()),
        None => (None, first.to_string()),
    }
}

/// Builds the address for a place. The theme is always written, so that what
/// is in the bar is the whole instruction and copying it carries everything.
pub fn build(theme: Theme, page: &str, anchor: Option<&str>) -> String {
    let mut out = String::from("/");
    out.push_str(theme.slug());
    if !page.is_empty() {
        out.push('/');
        out.push_str(page);
    }
    if let Some(anchor) = anchor.filter(|a| !a.is_empty()) {
        out.push('#');
        out.push_str(anchor);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_root_names_nothing() {
        assert_eq!(split("/"), (None, String::new()));
        assert_eq!(split(""), (None, String::new()));
    }

    #[test]
    fn a_theme_on_its_own_is_the_front_page_in_that_theme() {
        assert_eq!(split("/alien"), (Some(Theme::Alien), String::new()));
        assert_eq!(split("/lite"), (Some(Theme::Lite), String::new()));
    }

    #[test]
    fn a_theme_and_a_page() {
        assert_eq!(split("/lite/philosophy"), (Some(Theme::Lite), "philosophy".into()));
        assert_eq!(split("/alien/credits"), (Some(Theme::Alien), "credits".into()));
    }

    /// A first segment that is not a theme is a page. Somebody typing the
    /// short thing should still arrive.
    #[test]
    fn a_page_without_a_theme_still_names_the_page() {
        assert_eq!(split("/philosophy"), (None, "philosophy".into()));
        assert_eq!(split("/credits"), (None, "credits".into()));
    }

    #[test]
    fn extra_slashes_are_not_an_error() {
        assert_eq!(split("/lite/philosophy/"), (Some(Theme::Lite), "philosophy".into()));
        assert_eq!(split("//alien//credits//"), (Some(Theme::Alien), "credits".into()));
    }

    #[test]
    fn building_says_the_theme_every_time() {
        assert_eq!(build(Theme::Alien, "", None), "/alien");
        assert_eq!(build(Theme::Lite, "philosophy", None), "/lite/philosophy");
        assert_eq!(build(Theme::Alien, "credits", Some("license")), "/alien/credits#license");
        assert_eq!(build(Theme::Lite, "", Some("")), "/lite");
    }

    /// What is built has to come back apart into what built it.
    #[test]
    fn a_built_address_reads_back_the_same() {
        for theme in [Theme::Alien, Theme::Lite] {
            for page in ["", "philosophy", "credits"] {
                let (read_theme, read_page) = split(&build(theme, page, None));
                assert_eq!(read_theme, Some(theme));
                assert_eq!(read_page, page);
            }
        }
    }
}
