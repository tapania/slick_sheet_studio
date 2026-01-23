//! Default data generators for templates

use super::schema::{ContactInfo, Section, SlickSheetData, Stat, StyleHints};

/// Get default data for a template by ID
pub fn default_data_for_template(template_id: &str) -> SlickSheetData {
    match template_id {
        "product-sheet" => default_product_sheet(),
        "event-flyer" => default_event_flyer(),
        "one-pager" => default_one_pager(),
        "comparison-chart" => default_comparison_chart(),
        "case-study" => default_case_study(),
        "team-profile" => default_team_profile(),
        "pricing-table" => default_pricing_table(),
        "newsletter" => default_newsletter(),
        "infographic" => default_infographic(),
        "minimal" => default_minimal(),
        _ => default_minimal(),
    }
}

fn default_product_sheet() -> SlickSheetData {
    SlickSheetData::new("Product Name")
        .with_subtitle("Tagline goes here")
        .with_body("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.")
        .with_section(Section::list(
            "Key Features",
            vec![
                "Feature one with benefit".to_string(),
                "Feature two with benefit".to_string(),
                "Feature three with benefit".to_string(),
                "Feature four with benefit".to_string(),
            ],
        ))
        .with_section(Section::table(
            "Specifications",
            vec![
                vec!["Dimension".to_string(), "Value here".to_string()],
                vec!["Weight".to_string(), "Value here".to_string()],
                vec!["Material".to_string(), "Value here".to_string()],
                vec!["Warranty".to_string(), "Value here".to_string()],
            ],
            2,
        ))
        .with_contact(ContactInfo::with_email("sales@example.com"))
        .with_style(StyleHints {
            primary_color: Some("#e94560".to_string()),
            ..Default::default()
        })
}

fn default_event_flyer() -> SlickSheetData {
    SlickSheetData::new("EVENT NAME")
        .with_subtitle("Join Us For Something Amazing")
        .with_body("Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam.")
        .with_section(Section::list(
            "What to Expect",
            vec![
                "Keynote presentations from industry leaders".to_string(),
                "Networking opportunities".to_string(),
                "Interactive workshops".to_string(),
                "Refreshments provided".to_string(),
            ],
        ))
        .with_contact(ContactInfo {
            email: Some("events@example.com".to_string()),
            ..Default::default()
        })
        .with_style(StyleHints {
            primary_color: Some("#1a1a2e".to_string()),
            accent_color: Some("#e94560".to_string()),
            ..Default::default()
        })
}

fn default_one_pager() -> SlickSheetData {
    SlickSheetData::new("Company Name")
        .with_section(Section::text(
            "The Problem",
            "Organizations struggle with X, leading to Y consequences. Current solutions are inadequate because Z.",
        ))
        .with_section(Section::text(
            "Our Solution",
            "We provide a revolutionary approach that addresses these challenges.",
        ))
        .with_section(Section::list(
            "Why Choose Us",
            vec![
                "Proven track record with Fortune 500 clients".to_string(),
                "Award-winning customer support".to_string(),
                "Flexible pricing options".to_string(),
            ],
        ))
        .with_stat(Stat::new("95%", "Customer Satisfaction").with_color("#e94560"))
        .with_stat(Stat::new("2x", "Faster Results").with_color("#e94560"))
        .with_stat(Stat::new("$1M+", "Savings Generated").with_color("#e94560"))
        .with_contact(ContactInfo::with_email("hello@example.com"))
}

fn default_comparison_chart() -> SlickSheetData {
    SlickSheetData::new("Feature Comparison")
        .with_subtitle("See how we stack up against the competition")
        .with_section(Section::table(
            "Features",
            vec![
                vec![
                    "Feature".to_string(),
                    "Us".to_string(),
                    "Competitor A".to_string(),
                    "Competitor B".to_string(),
                ],
                vec![
                    "24/7 Support".to_string(),
                    "Yes".to_string(),
                    "Limited".to_string(),
                    "No".to_string(),
                ],
                vec![
                    "Cloud Storage".to_string(),
                    "Unlimited".to_string(),
                    "10 GB".to_string(),
                    "5 GB".to_string(),
                ],
                vec![
                    "API Access".to_string(),
                    "Yes".to_string(),
                    "Yes".to_string(),
                    "No".to_string(),
                ],
            ],
            4,
        ))
        .with_section(Section::list(
            "Summary",
            vec![
                "Unlimited cloud storage vs limited options elsewhere".to_string(),
                "Advanced analytics for data-driven decisions".to_string(),
                "Full API access for seamless integrations".to_string(),
            ],
        ))
}

fn default_case_study() -> SlickSheetData {
    SlickSheetData::new("Client Success Story")
        .with_subtitle("Industry: Technology | Company Size: Enterprise")
        .with_section(Section::list(
            "The Challenge",
            vec![
                "Legacy systems causing inefficiencies".to_string(),
                "Manual processes prone to errors".to_string(),
                "Lack of real-time visibility into operations".to_string(),
                "Growing costs with diminishing returns".to_string(),
            ],
        ))
        .with_section(Section::text(
            "Our Solution",
            "We implemented a comprehensive solution including thorough analysis and staged rollout.",
        ))
        .with_section(Section::quote(
            "Testimonial",
            "Working with this team transformed our operations. The results exceeded our expectations. — CEO, Client XYZ",
        ))
        .with_stat(Stat::new("40%", "Cost Reduction").with_color("#4ecca3"))
        .with_stat(Stat::new("60%", "Efficiency Gain").with_color("#4ecca3"))
        .with_stat(Stat::new("3x", "ROI").with_color("#4ecca3"))
}

fn default_team_profile() -> SlickSheetData {
    SlickSheetData::new("Meet Our Team")
        .with_subtitle("The experts behind our success")
        .with_section(Section::text(
            "Jane Smith",
            "CEO & Founder - 15+ years of industry experience. Previously led teams at Fortune 500 companies.",
        ))
        .with_section(Section::text(
            "John Doe",
            "CTO - Expert in scalable systems. Built platforms serving millions of users.",
        ))
        .with_section(Section::text(
            "Emily Chen",
            "Head of Design - Award-winning designer with a passion for user-centered design.",
        ))
        .with_section(Section::text(
            "Michael Brown",
            "Head of Sales - 10+ years in B2B sales. Track record of exceeding targets.",
        ))
        .with_contact(ContactInfo {
            website: Some("careers@example.com".to_string()),
            ..Default::default()
        })
}

fn default_pricing_table() -> SlickSheetData {
    SlickSheetData::new("Simple Pricing")
        .with_subtitle("Choose the plan that fits your needs")
        .with_section(Section::table(
            "Plans",
            vec![
                vec![
                    "Plan".to_string(),
                    "Price".to_string(),
                    "Features".to_string(),
                ],
                vec![
                    "Starter".to_string(),
                    "$9/month".to_string(),
                    "5 projects, 10 GB storage, Email support".to_string(),
                ],
                vec![
                    "Professional".to_string(),
                    "$29/month".to_string(),
                    "Unlimited projects, 100 GB storage, Priority support, API access".to_string(),
                ],
                vec![
                    "Enterprise".to_string(),
                    "Custom".to_string(),
                    "Everything in Pro, Unlimited storage, Dedicated support, SLA guarantee"
                        .to_string(),
                ],
            ],
            3,
        ))
        .with_style(StyleHints {
            primary_color: Some("#1a1a2e".to_string()),
            accent_color: Some("#e94560".to_string()),
            ..Default::default()
        })
}

fn default_newsletter() -> SlickSheetData {
    SlickSheetData::new("Monthly Newsletter")
        .with_subtitle("January 2024")
        .with_section(Section::text(
            "Featured Article",
            "The Future of Technology - Lorem ipsum dolor sit amet, consectetur adipiscing elit.",
        ))
        .with_section(Section::list(
            "Quick Updates",
            vec![
                "New Feature Launch - We are excited to announce our latest feature".to_string(),
                "Team Expansion - Welcome our newest team members".to_string(),
                "Upcoming Webinar - Register for our free webinar on Feb 1st".to_string(),
            ],
        ))
        .with_section(Section::list(
            "Upcoming Events",
            vec![
                "Jan 15 - Product Launch".to_string(),
                "Jan 22 - Customer Meetup".to_string(),
                "Feb 1 - Annual Conference".to_string(),
            ],
        ))
        .with_contact(ContactInfo {
            email: Some("hello@example.com".to_string()),
            phone: Some("(555) 123-4567".to_string()),
            website: Some("www.example.com".to_string()),
            ..Default::default()
        })
}

fn default_infographic() -> SlickSheetData {
    SlickSheetData::new("Industry Statistics 2024")
        .with_stat(Stat::new("78%", "Growth Rate").with_color("#4ecca3"))
        .with_stat(Stat::new("2.5M", "Active Users").with_color("#e94560"))
        .with_stat(Stat::new("$4.2B", "Market Size").with_color("#ffd93d"))
        .with_section(Section::list(
            "Market Trends",
            vec![
                "Cloud adoption: 80%".to_string(),
                "AI integration: 65%".to_string(),
                "Mobile-first: 45%".to_string(),
            ],
        ))
        .with_section(Section::list(
            "Regional Distribution",
            vec![
                "North America: 42%".to_string(),
                "Europe: 28%".to_string(),
                "Asia Pacific: 22%".to_string(),
                "Other: 8%".to_string(),
            ],
        ))
        .with_body(
            "The industry is experiencing unprecedented growth with technology adoption at an all-time high.",
        )
}

fn default_minimal() -> SlickSheetData {
    SlickSheetData::new("Title Here")
        .with_subtitle("Subtitle or tagline goes here")
        .with_body(
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.",
        )
        .with_section(Section::text(
            "Section One",
            "Sunt in culpa qui officia deserunt mollit anim id est laborum.",
        ))
        .with_section(Section::text(
            "Section Two",
            "Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit.",
        ))
        .with_contact(ContactInfo::with_email("hello@example.com"))
}

#[cfg(test)]
mod defaults_tests {
    use super::*;

    #[test]
    fn test_default_product_sheet() {
        let data = default_data_for_template("product-sheet");
        assert_eq!(data.title, "Product Name");
        assert!(data.subtitle.is_some());
        assert!(!data.sections.is_empty());
    }

    #[test]
    fn test_default_event_flyer() {
        let data = default_data_for_template("event-flyer");
        assert_eq!(data.title, "EVENT NAME");
    }

    #[test]
    fn test_unknown_template_returns_minimal() {
        let data = default_data_for_template("unknown-template");
        assert_eq!(data.title, "Title Here");
    }

    #[test]
    fn test_all_templates_have_defaults() {
        let template_ids = [
            "product-sheet",
            "event-flyer",
            "one-pager",
            "comparison-chart",
            "case-study",
            "team-profile",
            "pricing-table",
            "newsletter",
            "infographic",
            "minimal",
        ];

        for id in template_ids {
            let data = default_data_for_template(id);
            assert!(
                !data.title.is_empty(),
                "Template {} should have a non-empty title",
                id
            );
        }
    }
}
