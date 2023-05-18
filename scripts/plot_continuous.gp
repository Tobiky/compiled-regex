set term svg
set autoscale
unset log
unset label
set y2tic auto
unset title
set xlabel "n for a^n (int)"
set ylabel "time for a?^na^n (compiled) (ns)"
set y2label "time for a?^na^n 2 (interpreted) (ns)"
set output "_a__16a_16.svg"

plot "./condensed_data/continuous compiled _a__16a_16.csv" using 1:2 title "Compiled" with lines axes x1y1, \
     "./condensed_data/continuous interpreted _a__16a_16.csv" using 1:2 title "Interpreted" with lines axes x1y2
