+++
title = "Retriever"
sort_by = "weight"
weight = 1
template = "docs/page.html"

[extra]
toc = true
+++

Retrieve all the datasets for a given city.

## Goal

The goal is to download all the public datasets for a given city.

The input is a CSV file containing one or more city ratings entries.

### Remark

For 2021, there is about 11GB of datasets. Therefore retrieving them for a large
number of cities can take a while.

## Example

Given the following command:

```bash
retriever --from-csv examples/retriever/single_city_rating.csv \
  census-block connected-census-block data-dictionary overall-scores ways
```

And the following CSV file:

```csv
City,census_name,State,state_full,fips_code,Country,uuid,Year,Community Survey - Network,Community Survey - Awareness,Community Survey - Safety,Community Survey - Ridership,Community Score - Total,"Community Score - Total, Rounded",Community Survey - Responses,BNA - neighborhoods,BNA - opportunity_employment,BNA - opportunity_k12_education,BNA - opportunity_technical_vocational_college,BNA - opportunity_higher_education,BNA - opportunity,BNA - essential_services_doctors,BNA - essential_services_dentists,BNA - essential_services_hospitals,BNA - essential_services_pharmacies,BNA - essential_services_grocery,BNA - essential_services_social_services,BNA - essential_services,BNA - retail,BNA - recreation_parks,BNA - recreation_trails,BNA - recreation_community_centers,BNA - recreation,BNA - transit,BNA - overall_score,total_low_stress_miles,total_high_stress_miles,city_ratings_total,city_ratings_rounded,population,rank_country,pop_size,rank_country_size,rank_state_size,speed_limit,region,latitude,longitude
Pueblo,Pueblo city,CO,Colorado,862000,United States,ffc8c95c-bcbc-4587-81d8-2d8ff3033453,2021,45.996,43.322,49.27,75.946,53.6335,54,77,6.82,4.91,6.93,1.15,0,5.32,1.33,0,3.68,2.95,3.34,2.87,2.56,1.77,11.37,5.28,0.55,6.53,0.09,3.86,101.8,1107.1,13.8147,14,110841,624,medium,278,12,30,Mountain,38.271321,-104.610844
```

The following datasets will be downloaded:

```bash
.
├── United_States-CO-Pueblo-BNA.Data.Dictionary.xlsx
├── United_States-CO-Pueblo-neighborhood_census_blocks.zip
├── United_States-CO-Pueblo-neighborhood_connected_census_blocks.csv.zip
├── United_States-CO-Pueblo-neighborhood_overall_scores.csv
└── United_States-CO-Pueblo-neighborhood_ways.zip
```
