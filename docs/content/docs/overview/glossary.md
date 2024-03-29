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

**BNA Score**. A score out of 100 points representing the quality and
connectivity of the bike network in a given area (typically a city) generated by
the BNA.

**BNA Mechanics**. Open-source working group, including volunteers

**Brochure**. A visual document that can be distributed as an image (usually PNG
or SVG file), a PDF file or a print.

**City Ratings**. A PeopleForBikes program that measures the quality of bike
infrastructure in cities.

**City rating**. A city’s overall score in the City Ratings program, out of 100
possible points. For U.S. cities, the city rating consists of two inputs: The
Community Survey Score, worth 20% of the total, and the Bicycle Network Analysis
Score, worth 80% of the total score. For international cities, the city rating
may or may not include a Community Survey Score. If not, the city rating is
equivalent to the BNA Score.

**City Snapshot**. A survey fielded by PeopleForBikes in which respondents,
typically city transportation staff, provide input to inform the Bicycle Network
Analysis and City Ratings, such as local speed limits and initiatives to improve
bike infrastructure. The City Snapshot is not scored.

**Community Survey**. An opinion survey fielded by PeopleForBikes that measures
how people feel about bicycling in their city. Anyone can respond for a city
that they live in, work in, or frequently visit.

**Community Survey Score**. An aggregate score for a city out of 100 points
based on responses to the Community Survey.

**Rank**. A city’s numerical place among all cities rated (can be subdivided by
country, region, state, population size), e.g. 4 out of 158 small U.S. cities

**Scorecard**. A specific kind of brochure that summarizes City Ratings data for
each rated city.

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

**City Rating Entry**. A series of values compiled by the city ratings program
for a specific city. Each value is a score out of 100 for a category like
"employment", or "retail" for instance.
