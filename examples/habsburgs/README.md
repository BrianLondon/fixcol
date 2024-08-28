This is an example of the `fixed` paring and serialization library. In this
example we read a custom, column delimited data file into memory, run a computation
on that data and then write a result set out in a different, column delimited data file.

The basic job flow is in `main.rs`. The specifics of the computation are contained in
`alg.rs` but are not relevant to the example use of the serialization library.

## Input format

There are two types of records in the data file: person records and relation records.
The first column of the data file indicates the record type. A `P` indicates a 
person record and an `R` indicates a relation record.

### Person records

A person record is laid out as follows:

```text
P003    Charles V   1500-1558 -- example record
PAAA BBBBBBBBBB CCCCDDDD-EEEE -- coding for explanation below
```

- `P` is a literal `P` and indicates a person record
- `AAA` is a three digit, zero padded, right aligned number which serves as a
  record identifier
- `B...B` is the name of the person, right aligned
- `CCCC` is the regnal number (e.g., *V* in *Charles V*). It is left aligned and
  four columns, supporting roman numerals up to XXII.
- Life span is interpreted as two separate columns with a `-` between them.
  - `DDDD` is the year of birth
  - `EEEE` is the year of death

#### Note

In order to separate the birth and death years we need to disable **strict* mode.
Otherwise the non-whitespace character between those two columns would trigger
an error.

### Relation Records

A relation record describes a relationship between two person records.

```text
R PC 1  5   -- example record
R XX YYYZZZ -- coding for explanation below
```

- `R` is a literal `R` and indicates a relation record
- `XX` a two letter code indicating the relationship type. It has two legal values:
  - `SP` a marriage relationship
  - `PC` a parent-child relationship
- `YYY` the left member of the relationship (partner 1 or parent) as a left aligned, up to three digit number
- `ZZZ` the right member of the relationship (partner 2 or child) as a left aligned, up to three digit number

## Output Format

The output format is comparatively simple. It consists of a six column floating
point number (holding the COI), followed by a space and the person's name, left
aligned. The serialization format is defined on the `OutputRecord` struct.