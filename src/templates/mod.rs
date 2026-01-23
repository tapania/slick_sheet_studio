//! Templates module - Built-in templates for slick sheets
//!
//! This module provides:
//! - 10 curated templates for different use cases
//! - Template gallery for browsing and selection
//! - Template categories for organization

// Allow unused items that are part of the public API for testing/future use
#![allow(dead_code)]

#[cfg(test)]
mod tests;

/// Template category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TemplateCategory {
    /// Marketing materials (product sheets, one-pagers)
    Marketing,
    /// Business documents (case studies, pricing)
    Business,
    /// Event materials (flyers, invitations)
    Event,
    /// Data presentation (charts, infographics)
    Data,
    /// Simple, typography-focused designs
    Minimal,
}

impl TemplateCategory {
    /// Get the display name for the category
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Marketing => "Marketing",
            Self::Business => "Business",
            Self::Event => "Event",
            Self::Data => "Data",
            Self::Minimal => "Minimal",
        }
    }
}

/// A template definition
#[derive(Debug, Clone)]
pub struct Template {
    /// Unique identifier
    pub id: &'static str,
    /// Display name
    pub name: &'static str,
    /// Short description
    pub description: &'static str,
    /// Category for organization
    pub category: TemplateCategory,
    /// Optional preview SVG (base64 encoded)
    pub preview_svg: Option<&'static str>,
    /// Typst source code
    pub source: &'static str,
}

/// All available templates
pub static TEMPLATES: &[Template] = &[
    // 1. Product Sheet
    Template {
        id: "product-sheet",
        name: "Product Sheet",
        description: "Single product showcase with specifications",
        category: TemplateCategory::Marketing,
        preview_svg: None,
        source: r##"#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "Inter", size: 11pt)

#align(center)[
  #link("cmd://edit/title")[#text(size: 24pt, weight: "bold")[Product Name]]
  #v(0.5em)
  #link("cmd://edit/subtitle")[#text(size: 14pt, fill: gray)[Tagline goes here]]
]

#v(1em)

#columns(2, gutter: 2em)[
  == Overview
  #link("cmd://edit/body")[Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.]

  #colbreak()

  == Key Features
  - Feature one with benefit
  - Feature two with benefit
  - Feature three with benefit
  - Feature four with benefit
]

#v(1em)

== Specifications

#table(
  columns: (1fr, 2fr),
  stroke: 0.5pt + gray,
  inset: 8pt,
  [*Dimension*], [Value here],
  [*Weight*], [Value here],
  [*Material*], [Value here],
  [*Warranty*], [Value here],
)

#v(1em)

#align(center)[
  #rect(fill: rgb("#e94560"), radius: 4pt, inset: 12pt)[
    #text(fill: white, weight: "bold")[Contact Us: sales at example.com]
  ]
]
"##,
    },
    // 2. Event Flyer
    Template {
        id: "event-flyer",
        name: "Event Flyer",
        description: "Date, time, and location focused event announcement",
        category: TemplateCategory::Event,
        preview_svg: None,
        source: r##"#set page(width: 8.5in, height: 11in, margin: 0.5in)
#set text(font: "Inter", size: 11pt)

#align(center)[
  #rect(fill: rgb("#1a1a2e"), width: 100%, inset: 2em)[
    #link("cmd://edit/title")[#text(fill: white, size: 32pt, weight: "bold")[EVENT NAME]]
    #v(0.5em)
    #link("cmd://edit/subtitle")[#text(fill: rgb("#e94560"), size: 18pt)[Join Us For Something Amazing]]
  ]
]

#v(2em)

#align(center)[
  #grid(
    columns: 3,
    column-gutter: 2em,
    [
      #text(size: 14pt, weight: "bold")[DATE]
      #v(0.5em)
      January 15, 2024
    ],
    [
      #text(size: 14pt, weight: "bold")[TIME]
      #v(0.5em)
      7:00 PM - 10:00 PM
    ],
    [
      #text(size: 14pt, weight: "bold")[LOCATION]
      #v(0.5em)
      Main Conference Hall
    ],
  )
]

#v(2em)

== About This Event

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam.

#v(1em)

== What to Expect

- Keynote presentations from industry leaders
- Networking opportunities
- Interactive workshops
- Refreshments provided

#v(2em)

#align(center)[
  #text(size: 16pt, weight: "bold")[RSVP: events at example.com]
]
"##,
    },
    // 3. One-Pager
    Template {
        id: "one-pager",
        name: "One-Pager",
        description: "Executive summary style document",
        category: TemplateCategory::Marketing,
        preview_svg: None,
        source: r##"#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "Inter", size: 10pt)

#text(size: 24pt, weight: "bold")[Company Name]
#h(1fr)
#text(size: 10pt, fill: gray)[www.example.com]

#line(length: 100%, stroke: 0.5pt + gray)

#v(1em)

== The Problem

Organizations struggle with X, leading to Y consequences. Current solutions are inadequate because Z.

#v(0.5em)

== Our Solution

We provide a revolutionary approach that addresses these challenges through:

#grid(
  columns: 2,
  column-gutter: 1em,
  row-gutter: 0.5em,
  [*Innovation*], [Description of first benefit],
  [*Efficiency*], [Description of second benefit],
  [*Reliability*], [Description of third benefit],
)

#v(0.5em)

== Key Metrics

#table(
  columns: 3,
  stroke: none,
  inset: 6pt,
  align: center,
  [#text(size: 24pt, weight: "bold", fill: rgb("#e94560"))[95%]], [#text(size: 24pt, weight: "bold", fill: rgb("#e94560"))[2x]], [#text(size: 24pt, weight: "bold", fill: rgb("#e94560"))[\$1M+]],
  [Customer Satisfaction], [Faster Results], [Savings Generated],
)

#v(0.5em)

== Why Choose Us

- Proven track record with Fortune 500 clients
- Award-winning customer support
- Flexible pricing options

#v(1em)

#align(center)[
  #text(weight: "bold")[Ready to get started? Contact us at hello at example.com]
]
"##,
    },
    // 4. Comparison Chart
    Template {
        id: "comparison-chart",
        name: "Comparison Chart",
        description: "Side-by-side feature comparison",
        category: TemplateCategory::Data,
        preview_svg: None,
        source: r##"#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "Inter", size: 10pt)

#align(center)[
  #text(size: 24pt, weight: "bold")[Feature Comparison]
  #v(0.5em)
  #text(fill: gray)[See how we stack up against the competition]
]

#v(1.5em)

#table(
  columns: (2fr, 1fr, 1fr, 1fr),
  stroke: 0.5pt + gray,
  inset: 10pt,
  align: (left, center, center, center),

  table.header(
    [*Feature*],
    [*Us*],
    [*Competitor A*],
    [*Competitor B*],
  ),

  [24/7 Support], [Yes], [Limited], [No],
  [Cloud Storage], [Unlimited], [10 GB], [5 GB],
  [API Access], [Yes], [Yes], [No],
  [Custom Branding], [Yes], [No], [No],
  [Analytics Dashboard], [Advanced], [Basic], [Basic],
  [Mobile App], [Yes], [Yes], [No],
  [Priority Updates], [Yes], [No], [No],
)

#v(1.5em)

== Summary

Our solution provides the most comprehensive feature set at a competitive price point. Key advantages include:

- *Unlimited cloud storage* vs limited options elsewhere
- *Advanced analytics* for data-driven decisions
- *Full API access* for seamless integrations

#v(1em)

#align(center)[
  #rect(fill: rgb("#e94560"), radius: 4pt, inset: 12pt)[
    #text(fill: white, weight: "bold")[Start Your Free Trial Today]
  ]
]
"##,
    },
    // 5. Case Study
    Template {
        id: "case-study",
        name: "Case Study",
        description: "Problem, solution, and results format",
        category: TemplateCategory::Business,
        preview_svg: None,
        source: r##"#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "Inter", size: 11pt)

#text(size: 12pt, fill: rgb("#e94560"), weight: "bold")[CASE STUDY]

#text(size: 24pt, weight: "bold")[Client Success Story]

#text(fill: gray)[Industry: Technology | Company Size: Enterprise]

#line(length: 100%, stroke: 0.5pt + gray)

#v(1em)

== The Challenge

Client XYZ faced significant challenges in their operations:

- Legacy systems causing inefficiencies
- Manual processes prone to errors
- Lack of real-time visibility into operations
- Growing costs with diminishing returns

#v(1em)

== Our Solution

We implemented a comprehensive solution including:

#grid(
  columns: 2,
  column-gutter: 2em,
  [
    === Phase 1: Assessment
    Thorough analysis of existing systems and identification of key pain points.
  ],
  [
    === Phase 2: Implementation
    Staged rollout with minimal disruption to daily operations.
  ],
)

#v(1em)

== The Results

#table(
  columns: 3,
  stroke: none,
  inset: 8pt,
  align: center,
  [#text(size: 28pt, weight: "bold", fill: rgb("#4ecca3"))[40%]],
  [#text(size: 28pt, weight: "bold", fill: rgb("#4ecca3"))[60%]],
  [#text(size: 28pt, weight: "bold", fill: rgb("#4ecca3"))[3x]],
  [Cost Reduction], [Efficiency Gain], [ROI],
)

#v(1em)

#rect(fill: rgb("#f5f5f5"), inset: 1em, radius: 4pt)[
  _"Working with this team transformed our operations. The results exceeded our expectations."_

  #align(right)[— CEO, Client XYZ]
]
"##,
    },
    // 6. Team Profile
    Template {
        id: "team-profile",
        name: "Team Profile",
        description: "Staff and team member highlights",
        category: TemplateCategory::Business,
        preview_svg: None,
        source: r##"#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "Inter", size: 11pt)

#align(center)[
  #text(size: 28pt, weight: "bold")[Meet Our Team]
  #v(0.5em)
  #text(fill: gray)[The experts behind our success]
]

#v(1.5em)

#grid(
  columns: 2,
  column-gutter: 2em,
  row-gutter: 1.5em,

  // Team Member 1
  rect(fill: rgb("#f5f5f5"), inset: 1em, radius: 4pt)[
    #text(size: 14pt, weight: "bold")[Jane Smith]
    #v(0.25em)
    #text(fill: rgb("#e94560"))[CEO & Founder]
    #v(0.5em)
    15+ years of industry experience. Previously led teams at Fortune 500 companies.
  ],

  // Team Member 2
  rect(fill: rgb("#f5f5f5"), inset: 1em, radius: 4pt)[
    #text(size: 14pt, weight: "bold")[John Doe]
    #v(0.25em)
    #text(fill: rgb("#e94560"))[CTO]
    #v(0.5em)
    Expert in scalable systems. Built platforms serving millions of users.
  ],

  // Team Member 3
  rect(fill: rgb("#f5f5f5"), inset: 1em, radius: 4pt)[
    #text(size: 14pt, weight: "bold")[Emily Chen]
    #v(0.25em)
    #text(fill: rgb("#e94560"))[Head of Design]
    #v(0.5em)
    Award-winning designer with a passion for user-centered design.
  ],

  // Team Member 4
  rect(fill: rgb("#f5f5f5"), inset: 1em, radius: 4pt)[
    #text(size: 14pt, weight: "bold")[Michael Brown]
    #v(0.25em)
    #text(fill: rgb("#e94560"))[Head of Sales]
    #v(0.5em)
    10+ years in B2B sales. Track record of exceeding targets.
  ],
)

#v(1.5em)

#align(center)[
  #text(size: 14pt, weight: "bold")[Join Our Team]
  #v(0.5em)
  We are always looking for talented individuals. Visit careers at example.com
]
"##,
    },
    // 7. Pricing Table
    Template {
        id: "pricing-table",
        name: "Pricing Table",
        description: "Tiered pricing display",
        category: TemplateCategory::Business,
        preview_svg: None,
        source: r##"#set page(width: 8.5in, height: 11in, margin: 0.75in)
#set text(font: "Inter", size: 11pt)

#align(center)[
  #text(size: 28pt, weight: "bold")[Simple Pricing]
  #v(0.5em)
  #text(fill: gray)[Choose the plan that fits your needs]
]

#v(1.5em)

#grid(
  columns: 3,
  column-gutter: 1em,

  // Starter Plan
  rect(stroke: 1pt + gray, inset: 1.5em, radius: 4pt)[
    #align(center)[
      #text(size: 16pt, weight: "bold")[Starter]
      #v(0.5em)
      #text(size: 32pt, weight: "bold")[\$9]
      #text(fill: gray)[/month]
      #v(1em)
    ]
    - 5 projects
    - 10 GB storage
    - Email support
    - Basic analytics
    #v(1em)
    #align(center)[
      #rect(stroke: 1pt + gray, inset: 8pt, radius: 4pt)[
        Get Started
      ]
    ]
  ],

  // Professional Plan (highlighted)
  rect(fill: rgb("#1a1a2e"), inset: 1.5em, radius: 4pt)[
    #align(center)[
      #text(fill: rgb("#e94560"), size: 12pt, weight: "bold")[MOST POPULAR]
      #v(0.5em)
      #text(fill: white, size: 16pt, weight: "bold")[Professional]
      #v(0.5em)
      #text(fill: white, size: 32pt, weight: "bold")[\$29]
      #text(fill: gray)[/month]
      #v(1em)
    ]
    #text(fill: white)[
      - Unlimited projects
      - 100 GB storage
      - Priority support
      - Advanced analytics
      - API access
    ]
    #v(1em)
    #align(center)[
      #rect(fill: rgb("#e94560"), inset: 8pt, radius: 4pt)[
        #text(fill: white)[Get Started]
      ]
    ]
  ],

  // Enterprise Plan
  rect(stroke: 1pt + gray, inset: 1.5em, radius: 4pt)[
    #align(center)[
      #text(size: 16pt, weight: "bold")[Enterprise]
      #v(0.5em)
      #text(size: 32pt, weight: "bold")[Custom]
      #text(fill: gray)[pricing]
      #v(1em)
    ]
    - Everything in Pro
    - Unlimited storage
    - Dedicated support
    - Custom integrations
    - SLA guarantee
    #v(1em)
    #align(center)[
      #rect(stroke: 1pt + gray, inset: 8pt, radius: 4pt)[
        Contact Sales
      ]
    ]
  ],
)

#v(1.5em)

#align(center)[
  #text(fill: gray)[All plans include a 14-day free trial. No credit card required.]
]
"##,
    },
    // 8. Newsletter
    Template {
        id: "newsletter",
        name: "Newsletter",
        description: "Multi-section content layout",
        category: TemplateCategory::Marketing,
        preview_svg: None,
        source: r##"#set page(width: 8.5in, height: 11in, margin: 0.5in)
#set text(font: "Inter", size: 10pt)

#rect(fill: rgb("#1a1a2e"), width: 100%, inset: 1em)[
  #text(fill: white, size: 20pt, weight: "bold")[Monthly Newsletter]
  #h(1fr)
  #text(fill: gray)[January 2024]
]

#v(1em)

#columns(2, gutter: 1.5em)[

  == Featured Article

  #text(size: 14pt, weight: "bold")[The Future of Technology]

  Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.

  Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

  #text(fill: rgb("#e94560"))[Read more →]

  #v(1em)

  == Quick Updates

  *New Feature Launch*
  We are excited to announce our latest feature that will transform your workflow.

  *Team Expansion*
  Welcome our newest team members joining this month.

  *Upcoming Webinar*
  Register for our free webinar on Feb 1st.

  #colbreak()

  == Industry News

  #rect(fill: rgb("#f5f5f5"), inset: 0.75em, radius: 4pt)[
    *Market Trends*
    #v(0.25em)
    Key insights from this month's industry reports.
  ]

  #v(0.5em)

  #rect(fill: rgb("#f5f5f5"), inset: 0.75em, radius: 4pt)[
    *Regulatory Updates*
    #v(0.25em)
    Important changes affecting our sector.
  ]

  #v(1em)

  == Upcoming Events

  - *Jan 15* - Product Launch
  - *Jan 22* - Customer Meetup
  - *Feb 1* - Annual Conference

  #v(1em)

  == Contact Us

  Email: hello at example.com
  Phone: (555) 123-4567
  Web: www.example.com

]

#v(1em)

#line(length: 100%, stroke: 0.5pt + gray)

#align(center)[
  #text(fill: gray, size: 9pt)[
    You're receiving this because you subscribed to our newsletter.
  ]
]
"##,
    },
    // 9. Infographic
    Template {
        id: "infographic",
        name: "Infographic",
        description: "Data visualization focused layout",
        category: TemplateCategory::Data,
        preview_svg: None,
        source: r##"#set page(width: 8.5in, height: 11in, margin: 0.5in)
#set text(font: "Inter", size: 10pt)

#align(center)[
  #rect(fill: rgb("#e94560"), width: 100%, inset: 1em)[
    #text(fill: white, size: 24pt, weight: "bold")[Industry Statistics 2024]
  ]
]

#v(1em)

#grid(
  columns: 3,
  column-gutter: 1em,
  row-gutter: 1em,

  // Stat 1
  rect(fill: rgb("#1a1a2e"), inset: 1em, radius: 8pt)[
    #align(center)[
      #text(fill: rgb("#4ecca3"), size: 36pt, weight: "bold")[78%]
      #v(0.25em)
      #text(fill: white)[Growth Rate]
    ]
  ],

  // Stat 2
  rect(fill: rgb("#1a1a2e"), inset: 1em, radius: 8pt)[
    #align(center)[
      #text(fill: rgb("#e94560"), size: 36pt, weight: "bold")[2.5M]
      #v(0.25em)
      #text(fill: white)[Active Users]
    ]
  ],

  // Stat 3
  rect(fill: rgb("#1a1a2e"), inset: 1em, radius: 8pt)[
    #align(center)[
      #text(fill: rgb("#ffd93d"), size: 36pt, weight: "bold")[\$4.2B]
      #v(0.25em)
      #text(fill: white)[Market Size]
    ]
  ],
)

#v(1.5em)

== Key Findings

#grid(
  columns: 2,
  column-gutter: 2em,

  [
    === Market Trends
    #v(0.5em)
    #rect(fill: rgb("#4ecca3"), width: 80%, height: 8pt, radius: 4pt)[]
    Cloud adoption: 80%

    #rect(fill: rgb("#e94560"), width: 65%, height: 8pt, radius: 4pt)[]
    AI integration: 65%

    #rect(fill: rgb("#ffd93d"), width: 45%, height: 8pt, radius: 4pt)[]
    Mobile-first: 45%
  ],

  [
    === Regional Distribution
    #v(0.5em)
    - *North America*: 42%
    - *Europe*: 28%
    - *Asia Pacific*: 22%
    - *Other*: 8%
  ],
)

#v(1.5em)

#rect(fill: rgb("#f5f5f5"), width: 100%, inset: 1em, radius: 4pt)[
  #align(center)[
    #text(size: 14pt, weight: "bold")[Bottom Line]
    #v(0.5em)
    The industry is experiencing unprecedented growth with technology adoption at an all-time high.
  ]
]

#v(1em)

#align(center)[
  #text(fill: gray, size: 9pt)[Source: Industry Research Report 2024 | www.example.com]
]
"##,
    },
    // 10. Minimal
    Template {
        id: "minimal",
        name: "Minimal",
        description: "Clean, typography-focused design",
        category: TemplateCategory::Minimal,
        preview_svg: None,
        source: r##"#set page(width: 8.5in, height: 11in, margin: 1in)
#set text(font: "Inter", size: 12pt)
#set par(leading: 0.8em)

#v(2em)

#link("cmd://edit/title")[#text(size: 36pt, weight: "bold")[Title Here]]

#v(1em)

#link("cmd://edit/subtitle")[#text(fill: gray)[Subtitle or tagline goes here]]

#v(3em)

#link("cmd://edit/body")[Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.]

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident.

#v(2em)

== Section One

Sunt in culpa qui officia deserunt mollit anim id est laborum. Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium.

#v(1em)

== Section Two

Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt.

#v(3em)

#line(length: 30%, stroke: 1pt + gray)

#v(1em)

#text(fill: gray)[
  Contact: hello at example.com
]
"##,
    },
];

/// Template gallery for browsing and selecting templates
#[derive(Debug)]
pub struct TemplateGallery {
    templates: &'static [Template],
}

impl Default for TemplateGallery {
    fn default() -> Self {
        Self::new()
    }
}

impl TemplateGallery {
    /// Create a new template gallery with all built-in templates
    pub fn new() -> Self {
        Self {
            templates: TEMPLATES,
        }
    }

    /// Get all templates
    pub fn templates(&self) -> &[Template] {
        self.templates
    }

    /// Get a template by ID
    pub fn get(&self, id: &str) -> Option<&Template> {
        self.templates.iter().find(|t| t.id == id)
    }

    /// Get templates by category
    pub fn by_category(&self, category: TemplateCategory) -> Vec<&Template> {
        self.templates
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Get all unique categories
    pub fn categories(&self) -> Vec<TemplateCategory> {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        self.templates
            .iter()
            .filter_map(|t| {
                if seen.insert(t.category) {
                    Some(t.category)
                } else {
                    None
                }
            })
            .collect()
    }
}
