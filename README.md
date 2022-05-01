# rust-log-monitor

Useful if you have a log file that is being written to and you want to tail it.
`read()` will read from the beginning until the last line in the log.
Then `read()` will read from that point to the end of the log if any new lines are written to the log.
So you can keep calling `read()` periodically.

If you want to read a limited number of lines then you can call `read_n_lines(n)` instead and it will operate the same as `read()`.

`next_line()` will read the next line.


`read_range(from, to, force)` will read a range of lines. Force will indicate
if you have alread read past the `from` line, if the file should be reopened
and we should start reading from the start until we get to the `from` line.

`from` and `to` can be negative numbers, where `-1` is the last line, and `-2`
is the second last line. So `from=-1` `to=-1` will read the last line in the log file.

The `to` line is inclusive. The `from` line starts at line number 1 (not zero based).

If `from` is 0 (zero), that indicates to start from the last read line.

`from=-1` `to=-1` `force=true` will reread the entire file if there was no additional line written since the last time it was called.

`from=-1` `to=-1` `force=true` will return nothing if there was no additional lines written since the last time it was called, otherwise the last written line.

`from=0` `to=-1` `force=true` will return nothing if there was no additional lines written since the last time it was called, otherwise all the last lines since the last request.

