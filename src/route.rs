//! Turning an address into a place on the site, and back.
//!
//! An address holds both things a reader might want to send somebody: which
//! page, and which of the two themes it should be read in. `/lite/philosophy`
//! is a whole instruction.
//!
//! The theme comes first because it is the rarer thing to name and the easier
//! to strip: everything after it reads the same whichever theme is in front.
//! Nothing here touches the browser, so all of it can be checked.

/// The ways the site can look.
///
/// `alien` is the site's own look, and what the name turns out to stand for.
/// `silver` is the same place in a colder light — Ag is silver and XAG is what
/// a troy ounce of it trades under, which is a coincidence rather than the
/// point, so it is a theme and not the identity.
///
/// `lite` is the honest name for the other pair: no sky, no wider gamut, no
/// simulation running behind anything — Solarized and nothing else.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Alien,
    Silver,
    Lite,
    Lite2,
}

pub const THEMES: [Theme; 4] = [Theme::Alien, Theme::Silver, Theme::Lite, Theme::Lite2];

impl Theme {
    pub fn slug(self) -> &'static str {
        match self {
            Theme::Alien => "alien",
            Theme::Silver => "silver",
            Theme::Lite => "lite",
            Theme::Lite2 => "lite2",
        }
    }

    /// What it is called where a reader has to pick one.
    pub fn label(self) -> &'static str {
        match self {
            Theme::Alien => "Alien",
            Theme::Silver => "Silver",
            Theme::Lite => "Solarized Dark",
            Theme::Lite2 => "Solarized Light",
        }
    }

    /// What goes on the document. The alien one is the absence of an answer,
    /// which is what lets the stylesheet say "no theme asked for" with
    /// `:not([data-theme])` rather than listing everything it is not.
    pub fn attribute(self) -> Option<&'static str> {
        match self {
            Theme::Alien => None,
            Theme::Silver => Some("silver"),
            Theme::Lite => Some("solarized"),
            Theme::Lite2 => Some("solarized-light"),
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        THEMES.into_iter().find(|t| t.slug() == slug)
    }

    /// Whether the rocks are drifting behind it. Alien and Silver are the same
    /// place in two lights; the Solarized pair are flat by definition, and
    /// running a simulation behind a palette chosen for its calm would be
    /// missing the point of asking for it.
    pub fn has_sky(self) -> bool {
        matches!(self, Theme::Alien | Theme::Silver)
    }
}

/// Which theme a host arrives in when nothing else has said.
///
/// The two domains are the same site, but `.org` opens in silver and `.com` in
/// alien. It is only a default: an address that names a theme wins, and so does
/// a choice the reader has made before.
pub fn default_theme(host: &str) -> Theme {
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if host == "org" || host.ends_with(".org") {
        Theme::Silver
    } else {
        Theme::Alien
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
        assert_eq!(split("/lite2"), (Some(Theme::Lite2), String::new()));
        assert_eq!(split("/silver"), (Some(Theme::Silver), String::new()));
    }

    /// `lite` must not swallow `lite2`, which a prefix match would.
    #[test]
    fn the_two_lite_themes_are_told_apart() {
        assert_eq!(Theme::from_slug("lite"), Some(Theme::Lite));
        assert_eq!(Theme::from_slug("lite2"), Some(Theme::Lite2));
        assert_eq!(Theme::from_slug("lite3"), None);
        assert_eq!(split("/lite2/credits"), (Some(Theme::Lite2), "credits".into()));
    }

    /// Every theme has a slug of its own, and only the alien one is the
    /// absence of an attribute.
    #[test]
    fn each_theme_is_distinct() {
        let slugs: Vec<_> = THEMES.iter().map(|t| t.slug()).collect();
        let mut sorted = slugs.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), slugs.len(), "two themes share a slug");
        assert_eq!(
            THEMES.iter().filter(|t| t.attribute().is_none()).count(),
            1,
            "exactly one theme is the default"
        );
        assert_eq!(THEMES.iter().filter(|t| t.has_sky()).count(), 2);
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
    fn org_opens_in_silver_and_everything_else_in_alien() {
        assert_eq!(default_theme("xag-lang.org"), Theme::Silver);
        assert_eq!(default_theme("www.xag-lang.org"), Theme::Silver);
        assert_eq!(default_theme("XAG-LANG.ORG"), Theme::Silver);
        // A trailing dot is a fully qualified name, and still that host.
        assert_eq!(default_theme("xag-lang.org."), Theme::Silver);

        assert_eq!(default_theme("xag-lang.com"), Theme::Alien);
        assert_eq!(default_theme("www.xag-lang.com"), Theme::Alien);
        assert_eq!(default_theme("localhost"), Theme::Alien);
        assert_eq!(default_theme("127.0.0.1"), Theme::Alien);
        assert_eq!(default_theme(""), Theme::Alien);
    }

    /// `.org` has to be the end of the host and not merely in it, or
    /// `xag-lang.org.example.com` would open in the wrong one.
    #[test]
    fn org_has_to_be_the_end_of_the_host() {
        assert_eq!(default_theme("xag-lang.org.example.com"), Theme::Alien);
        assert_eq!(default_theme("orgs.example.com"), Theme::Alien);
        assert_eq!(default_theme("borg.example.com"), Theme::Alien);
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
        for theme in THEMES {
            for page in ["", "philosophy", "credits"] {
                let (read_theme, read_page) = split(&build(theme, page, None));
                assert_eq!(read_theme, Some(theme));
                assert_eq!(read_page, page);
            }
        }
    }
}
