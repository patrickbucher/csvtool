# CSV Tool

A tool to deal with CSV files.

## Sum Up Duration Column

Sums up columns of the form `hh:mm`.

    csvtool sum-duration --infile example.csv --outfile result.csv --column Effort

## Rewrite a CSV file

Runs a CSV file through parsing/writing (to clean up some cosmetic problems):

    csvtool rewrite --infile nasty.csv --outfile beautiful.csv
