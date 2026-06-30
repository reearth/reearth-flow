# (DRAFT) Regular Expression

`Regex` is a type object. `Regex(pattern)` constructs a regex object from the string `pattern`.

## pattern language

Patterns should support POSIX Extended Regular Expression (ERE) syntax, excluding locale-specific bracket constructs: collating symbols (`[.ch.]`) and equivalence classes (`[=a=]`).

A capture group is a sub-expression enclosed in parentheses `(...)`.

## find

For regex `r`, `r.find(s)` returns the first match of `r` in string `s`, or `null` if there is no match.

The form of the return value depends on the number of capture groups in `r`:

- No capture groups: the matched substring.
- One capture group: the text captured by that group, or `null` if the group did not participate.
- Multiple capture groups: a list of captured texts in definition order; a non-participating group yields `null` in its position.

## find_all

For regex `r`, `r.find_all(s)` returns a list of all non-overlapping matches of `r` in `s`, in left-to-right order. Returns an empty list if there are no matches.

Each element has the same form as a successful `find` result for that occurrence.

## split

For regex `r`, `r.split(s)` splits string `s` on each non-overlapping match of `r`, returning a list of the parts between matches.

`r.split(s, limit)` splits at most `limit` times, producing at most `limit + 1` parts.
