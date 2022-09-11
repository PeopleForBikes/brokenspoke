+++
title = "Shortcodes"
sort_by = "weight"
weight = 1
template = "docs/page.html"

[extra]
toc = true
+++

A tool to convert a City Ratings CSV file to a Shortcode CSV file.

Basically, the 2 files will be identical, except that the shortcode file headers
will be at most 5 character longs, without underscores nor white spaces.

## Goal

The goal is to simplify the templates that will be used when performing the
data-merge operation. If the templates had to deal with the full names, it would
be more difficult for the designers to put them together.

### Example

city_ratings.csv:

```csv
City,census_name,State,state_full,fips_code,Country,uuid,Year,Community Survey - Network,Community Survey - Awareness,Community Survey - Safety,Community Survey - Ridership,Community Score - Total,"Community Score - Total, Rounded",Community Survey - Responses,BNA - neighborhoods,BNA - opportunity_employment,BNA - opportunity_k12_education,BNA - opportunity_technical_vocational_college,BNA - opportunity_higher_education,BNA - opportunity,BNA - essential_services_doctors,BNA - essential_services_dentists,BNA - essential_services_hospitals,BNA - essential_services_pharmacies,BNA - essential_services_grocery,BNA - essential_services_social_services,BNA - essential_services,BNA - retail,BNA - recreation_parks,BNA - recreation_trails,BNA - recreation_community_centers,BNA - recreation,BNA - transit,BNA - overall_score,total_low_stress_miles,total_high_stress_miles,city_ratings_total,city_ratings_rounded,population,rank_country,pop_size,rank_country_size,rank_state_size,speed_limit,region,latitude,longitude
Pueblo,Pueblo city,CO,Colorado,862000,United States,ffc8c95c-bcbc-4587-81d8-2d8ff3033453,2021,45.996,43.322,49.27,75.946,53.6335,54,77,6.82,4.91,6.93,1.15,0,5.32,1.33,0,3.68,2.95,3.34,2.87,2.56,1.77,11.37,5.28,0.55,6.53,0.09,3.86,101.8,1107.1,13.8147,14,110841,624,medium,278,12,30,Mountain,38.271321,-104.610844
```

shortcodes.csv:

```csv
ci,co,st,uuid,po,ra,rasc,nw,aw,sf,rs,total,cssc,responses,nh,op,es,ret,rec,tr,bnasc,lsm,hsm
Pueblo,United States,CO,ffc8c95c-bcbc-4587-81d8-2d8ff3033453,110841,13.8147,14,46,43,49,76,54,54,77,7,5,3,2,7,0,4,102,255
```
