Small RSS scraping program.

You give it a list of blogs as jsonl:

```json
{"title":"Jade", "url":"jade.fyi", "feed":"/rss.xml"}
{"title":"Brendan Gregg", "url":"www.brendangregg.com/blog", "feed":"/rss.xml"}
{"title":"Serge Zaitsev", "url":"zserge.com", "feed":"/rss.xml"}
...
```
and run `rssfetch blogs.jsonl > posts.jsonl` to get a list of posts like:

``` json
{"title":"Oh no, `git send-email`", "url":"https://jade.fyi/blog/oh-no-git-send-email/", "blog_title":"Jade", "blog_url":"jade.fyi", "date":"2022-04-22"}
{"title":"Debugging: using rr to deal with unruly children (processes)", "url":"https://jade.fyi/blog/debugging-rr-children/", "blog_title":"Jade", "blog_url":"jade.fyi", "date":"2022-02-24"}
{"title":"On Transpilers", "url":"https://zserge.com/posts/transpilers/", "blog_title":"Serge Zaitsev", "blog_url":"zserge.com", "date":"2022-07-06"}
{"title":"Learn a language by writing too many Forths", "url":"https://zserge.com/posts/too-many-forths/", "blog_title":"Serge Zaitsev", "blog_url":"zserge.com", "date":"2022-07-05"}
...
```

On stderr it'll emit progress/status info:
```
✅ Serge Zaitsev: 78 posts
❌ Brendan Gregg: ParseError(NoFeedRoot)
✅ Jade: 62 posts
...
```

Some feeds only show the most recent N posts. If we want to get more than that then we'd have to keep around the posts list and update it with new entries on each run, but that'd also make things more complicated and I don't particularly care about that functionality.
