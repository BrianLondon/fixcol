This is an example of the `fixed` paring and serialization library. In this
example we read a custom, column delimited data file containing the 
[genealogy of Charles II of Spain](https://en.wikipedia.org/wiki/Habsburg_family_tree#Ancestors_of_Charles_II_of_Spain). 
Then we run a graph algorithm on the resulting data structure to calculate the
[COI](https://en.wikipedia.org/wiki/Coefficient_of_inbreeding) for each of the
people in that genealogy. Finally, we write an output file with that calculated
data.

## Input format

