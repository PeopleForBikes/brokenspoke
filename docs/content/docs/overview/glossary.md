+++
title = "Glossary"
date = 2021-11-26T08:20:00+00:00
weight = 20
template = "docs/page.html"

[extra]
toc = true
top = false
+++

In the BNA Mechanics world, there are a lot of concepts which use a lot of
words, some times even different ones to say the same thing. The goal of this
glossary is to reduce the confusion and improve the clarity of the vocabulary
used in the different projects.

---

**Bicycle Network Analysis (BNA)**. Software that analyzes the quality and
connectivity of bike infrastructure in cities.

**City Ratings**. A PeopleForBikes program that ranks cities annually based on
their ratings generated by the BNA.

**BNA Analysis**. A set of actions that is used to produce a Rating for a single
city.

**Submission**: A quadruplet consisting of a city name, state, country, and FIPS
code, provided to initiate an analysis in the BNA. A Submission can have three
approval statuses: Pending, Approved, or Rejected.

**Pipeline**: A set of actions being executed automatically in a specific order.

**BNA Pipeline**: Pipeline used to collect all the required input data to run a
BNA Analysis.

**BNA Output**. The set of files and data generated by the BNA Analysis for a
single city.

**Rating**. All input, output, and ancillary values associated with running the
BNA once for a city. One line in the historical data file represents a rating.

**BNA Component**. A subsection of the Rating generated by the BNA. There are
eight components of a Rating: Summary, Core Services, Infrastructure,
Opportunity, People, Recreation, Retail, and Transit.

**Rank**. A city’s numerical place among all cities rated in a given year based
on overall scores (can be subdivided by country, region, state, population
size), e.g. 4 out of 158 small U.S. cities.

**BNA Mechanics**. Open-source working group, including volunteers.

**Brochure**. A visual document that can be distributed as an image (usually PNG
or SVG file), a PDF file or a print.

**City Snapshot**. A feedback form on the City Ratings website that enables
people to provide input to inform the future analyses for their city, such as
local speed limits.

**Scorecard**. A specific kind of brochure that summarizes Rating data for each
rated city.

**SPRINT**. Acronym summarizing the main factors that impact the BNA Score.

- **S**afe Speeds
- **P**rotected Bike Lanes
- **R**eallocated Space
- **I**ntersection Treatments
- **N**etwork Connections
- **T**rusted Data

**Infrastructure**. Refers to any facility that is built to allow people to move
around using a bike or other micromobility device that typically travels under
20 mph (32 km/h).

**Pipeline**: A set of actions being executed automatically in a specific order.

**BNA pipeline**: Pipeline used to collect all the required input data to run a
BNA analysis.

### Incubating

> **WARNING: the terms bellow are still waiting to be defined properly. They may
> or may not be included in the glossary.**

**Shortcode**. A 2 to 4 letter code used in CSV file headers to simplify their
inclusion in templates during a data-merge operation, typically using SVGGloo.
The associated values may be rounded to be integers instead of decimals.

**ShortCityRatings**. A version of the **city ratings** where the headers are
shortcodes and the values are rounded to be integers instead of decimals.

**Shortscorecard**. A **scorecard** with the shortened headers and rounded
integer values.

**Template**. A file, typically an SVG file but it could be any text-based file
format, whose goal is to be combined with a shortcode CSV file during a
data-merge operation.