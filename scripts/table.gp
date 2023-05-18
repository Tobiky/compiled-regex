set term svg
set autoscale
unset log
unset label
unset title
set xlabel "RegEx (test data)"
set ylabel "time (ns)"
set output "table_2.svg"

set style data histogram
set style histogram cluster
set style fill solid

plot "./condensed_data/table.csv" every ::5::7 using 2:xtic(1) title "Compiled", \
     "" every ::5::7 using 3 title "Interpreted"

