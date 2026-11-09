## Visual verification

After conversion and before delivery, check the HTML in light and dark themes and
at phone width. Look for overflow, overlapping labels, unclosed `:::` markers, and
unreadable colors. Fix issues before sharing the output. If a browser cannot open
a large embedded video through `file://`, serve the output with
`python3 -m http.server 8000`. If visual inspection is unavailable, report it;
do not claim it was performed.
