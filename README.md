# compiled-regex

Thesis on performance benifits of compiling RegEx

## Datasets

These datasets are exlusively used for benchmarking purposes.

Datasets to test RegEx on

* [Keggle URL Dataset](https://www.kaggle.com/datasets/teseract/urldataset)

RegEx or parsable into RegEx

* [EasyList Advert Blocking List (only URL seciont)](https://easylist.to/easylist/easylist.txt)

## To do

* Optimizations
  * Generated Code
    * [ ] Character sequences replacing character concatenation
    ("ab" should be tried as "ab", not "a" then "b")
    * [ ] Byte automata instead of using `.chars().nth()`
    * [ ] Use `MIN_LEN` constant to boundry check strings
  * Compilation
    * [ ] Reduce string duplication/copying
