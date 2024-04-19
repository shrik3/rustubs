# Contribution Guidelines

This document contains guidelines for contributing code to rustubs. It has to be
followed in order for your patch to be approved and applied.



## Contribution Channels

Anyone can contribute to rustubs. First you need to clone the repository and build
the project:

    $ git clone https://git.sr.ht/~shrik3/rustubs
    $ cd rustubs
    $ make

Patch the code. Write some tests. Ensure that your code is properly formatted
with `rustfmt`, Notably this project uses hard tabs for indentation. Ensure that
everything builds and works as expected. Ensure that you did not break anything.

- Do not forget to update the docs.
- Run the linter (TODO)

Once you are happy with your work, you can create a commit (or several
commits). Follow these general rules:

- Limit the first line (title) of the commit message to 60 characters.
- Use a short prefix for the commit title for readability with `git log
  --oneline`. See recent commits for inspiration.
- Only use lower case letters for the commit title except when quoting symbols
  or known acronyms.
- Use the body of the commit message to actually explain what your patch does
  and why it is useful. Even if your patch is a one line fix, the description
  is not limited in length and may span over multiple paragraphs. Use proper
  English syntax, grammar and punctuation.
- Address only one issue/topic per commit.
- Describe your changes in imperative mood, e.g. *"make xyzzy do frotz"*
  instead of *"[This patch] makes xyzzy do frotz"* or *"[I] changed xyzzy to do
  frotz"*, as if you are giving orders to the codebase to change its behaviour.
- If you are fixing a ticket, use appropriate
  [commit trailers](https://man.sr.ht/git.sr.ht/#referencing-tickets-in-git-commit-messages).
- If you are fixing a regression introduced by another commit, add a `Fixes:`
  trailer with the commit id and its title.
- When in doubt, follow the format and layout of the recent existing commits.
- TODO: add changelog and guidelines.
- The following trailers are accepted in commits. If you are using multiple
  trailers in a commit, it's preferred to also order them according to this
  list. Note, that the `commit-msg` hook (see below for installing) will
  automatically sort them for you when committing.

  * `Closes: <URL>` closes the ticket with the neutral `CLOSED` resolution.
  * `Fixes: <URL>` closes the ticket with the `FIXED` resolution.
  * `Fixes: <sha> ("<title>")` reference the commit that introduced a regression.
  * `Implements: <URL>` closes the ticket with the `IMPLEMENTED` resolution.
  * `References: <URL>` adds a comment to the ticket.
  * `Link:`
  * `Changelog-added:`
  * `Changelog-fixed:`
  * `Changelog-changed:`
  * `Changelog-deprecated:`
  * `Cc:`
  * `Suggested-by:`
  * `Requested-by:`
  * `Reported-by:`
  * `Co-authored-by:`
  * `Signed-off-by:` compulsory!
  * `Tested-by:` used in review _after_ submission to the mailing list. If
    minimal changes occur between respins, feel free to include that into your
    respin to keep track of previous reviews.
  * `Reviewed-by:` used in review _after_ submission. If minimal changes occur
    between respins, feel free to include that into your respin to keep track
    of previous reviews.
  * `Acked-by:` used in review _after_ submission.

There is a great reference for commit messages in the
[Linux kernel documentation](https://www.kernel.org/doc/html/latest/process/submitting-patches.html#describe-your-changes).

**IMPORTANT: you must sign-off your work** using `git commit --signoff`. Follow
the [Linux kernel developer's certificate of origin][linux-signoff] for more
details. All contributions are made under the MIT license. If you do not want to
disclose your real name, you may sign-off using a pseudonym. Here is an example:

    Signed-off-by: Tianhao Wang <shrik3@mailbox.org>

Before sending the patch, you should configure your local clone with sane
defaults:

    $ gmake gitconfig
    git config format.subjectPrefix "PATCH rustubs"
    git config sendemail.to "~shrik3/rustubs@lists.sr.ht"
    git config format.notes true
    git config notes.rewriteRef refs/notes/commits
    git config notes.rewriteMode concatenate

And send the patch to the mailing list ([step-by-step
instructions][git-send-email-tutorial]):

    $ git send-email --annotate -1

TODO: add wiki.

Before your patch can be applied, it needs to be reviewed and approved by
others. They will indicate their approval by replying to your patch with
a [Tested-by, Reviewed-by or Acked-by][linux-review] (see also: [the git
wiki][git-trailers]) trailer. For example:

    Acked-by: Tianhao Wang <shrik3@mailbox.org>

There is no "chain of command" in rustubs. Anyone that feels comfortable enough
to "ack" or "review" a patch should express their opinion freely with an
official Acked-by or Reviewed-by trailer. If you only tested that a patch works
as expected but did not conduct a proper code review, you can indicate it with a
Tested-by trailer.

You can follow the review process via email and on the [web ui][web-ui].

Wait for feedback. Address comments and amend changes to your original commit.
Then you should send a v2 (and maybe a v3, v4, etc.):

    $ git send-email --annotate -v2 -1

Be polite, patient and address *all* of the reviewers' remarks. If you disagree
with something, feel free to discuss it.

To help reviewers track what changed between respins of your patch, it is nice
to include a mini change log **after** the `---` line that separates your
commit message from the diff. You can either do that manually when reviewing
(`git send-email --annotate`) before sending your email, or you can use [git
notes][git-notes] to make this part of your git workflow:

    $ git notes edit $ref

[git-notes]: https://git-scm.com/docs/git-notes

When `format.notes = true` is set in your git configuration, notes attached to
commits will automatically be included in the correct location by `git
format-patch` and `git send-email`.

If you have set `notes.rewriteMode = concatenate`, squashing commits together
with `git rebase -i` will also merge their respective notes by concatenating
them.

Once your patch has been reviewed and approved (and if the maintainer is OK
with it), it will be applied and pushed.

IMPORTANT: Do NOT use `--in-reply-to` when sending followup versions of a patch
set. It causes multiple versions of the same patch to be merged under v1 in the
[web ui][web-ui]

[web-ui]: https://lists.sr.ht/~shrik3/rustubs/patches

## Attributions

If the patch contains copied or derived works from others, you must make sure
the original work is
[compatible with EUPLv1.2](https://joinup.ec.europa.eu/collection/eupl/matrix-eupl-compatible-open-source-licences).
Make sure to add proper lines to `ATTRIBUTIONS` to comply their license and give
the original authors credits.

If some articles, tutorials or documentations help you to understand and solve
the problems, while not mandatory, please kindly refer to the source in the
`Documentation Sources` section of `README.md`.

## Code Style

Please refer only to the quoted sections when guidelines are sourced from
outside documents as some rules of the source material may conflict with other
rules set out in this document.

When updating an existing file, respect the existing coding style unless there
is a good reason not to do so.

### Indentation

We use *hard tabs* for indentations with tabwidth == 4 (TODO: consider using 8)

### Breaking long lines and strings

Wrapping rules follow the Linux kernel coding style:

> Coding style is all about readability and maintainability using commonly
> available tools.
>
> The preferred limit on the length of a single line is 80 columns.
>
> Statements longer than 80 columns should be broken into sensible chunks,
> unless exceeding 80 columns significantly increases readability and does not
> hide information.
> […]
> These same rules are applied to function headers with a long argument list.
>
> However, never break user-visible strings such as printk messages because
> that breaks the ability to grep for them.
> — [Linux kernel coding style][linux-coding-style]

Whether or not wrapping lines is acceptable can be discussed on IRC or the
mailing list, when in doubt.

### Functions

Function rules follow the Linux kernel coding style:

> Functions should be short and sweet, and do just one thing. They should fit
> on one or two screenfuls of text (the ISO/ANSI screen size is 80x24, as we
> all know), and do one thing and do that well.
>
> The maximum length of a function is inversely proportional to the complexity
> and indentation level of that function. So, if you have a conceptually simple
> function that is just one long (but simple) case-statement, where you have to
> do lots of small things for a lot of different cases, it’s OK to have
> a longer function.
>
> However, if you have a complex function, and you suspect that
> a less-than-gifted first-year high-school student might not even understand
> what the function is all about, you should adhere to the maximum limits all
> the more closely. Use helper functions with descriptive names (you can ask
> the compiler to in-line them if you think it’s performance-critical, and it
> will probably do a better job of it than you would have done).
>
> Another measure of the function is the number of local variables. They
> shouldn’t exceed 5-10, or you’re doing something wrong. Re-think the
> function, and split it into smaller pieces. A human brain can generally
> easily keep track of about 7 different things, anything more and it gets
> confused. You know you’re brilliant, but maybe you’d like to understand what
> you did 2 weeks from now.
> — [Linux kernel coding style][linux-coding-style]

### Commenting

Function rules follow the Linux kernel coding style:

> Comments are good, but there is also a danger of over-commenting. NEVER try
> to explain HOW your code works in a comment: it’s much better to write the
> code so that the working is obvious, and it’s a waste of time to explain
> badly written code.
>
> Generally, you want your comments to tell WHAT your code does, not HOW. Also,
> try to avoid putting comments inside a function body: if the function is so
> complex that you need to separately comment parts of it, you should probably
> go back to [the previous section regarding functions] for a while. You can
> make small comments to note or warn about something particularly clever (or
> ugly), but try to avoid excess. Instead, put the comments at the head of the
> function, telling people what it does, and possibly WHY it does it.
>
> When commenting […] API functions, please use the [GoDoc] format. See the
> [official documentation][godoc-comments] for details.
> — [Linux kernel coding style][linux-coding-style]

### Editor modelines

> Some editors can interpret configuration information embedded in source
> files, indicated with special markers. For example, emacs interprets lines
> marked like this:
>
>     -*- mode: c -*-
>
> Or like this:
>
>     /*
>     Local Variables:
>     compile-command: "gcc -DMAGIC_DEBUG_FLAG foo.c"
>     End:
>     */
>
> Vim interprets markers that look like this:
>
>     /* vim:set sw=8 noet */
>
> Do not include any of these in source files. People have their own personal
> editor configurations, and your source files should not override them. This
> includes markers for indentation and mode configuration. People may use
> their own custom mode, or may have some other magic method for making
> indentation work correctly.
> — [Linux kernel coding style][linux-coding-style]

In the same way, files specific to only your workflow (for example the `.idea`
or `.vscode` directory) are not desired. If a script might be useful to other
contributors, it can be sent as a separate patch that adds it to the `contrib`
directory. 

[git-send-email-tutorial]: https://git-send-email.io/
[git-trailers]: https://git.wiki.kernel.org/index.php/CommitMessageConventions
[godoc-comments]: https://go.dev/blog/godoc
[gofumpt-repo]: https://github.com/mvdan/gofumpt
[linux-coding-style]: https://www.kernel.org/doc/html/v5.19-rc8/process/coding-style.html
[linux-review]: https://www.kernel.org/doc/html/latest/process/submitting-patches.html#using-reported-by-tested-by-reviewed-by-suggested-by-and-fixes
[linux-signoff]: https://www.kernel.org/doc/html/latest/process/submitting-patches.html#sign-your-work-the-developer-s-certificate-of-origin
[scdoc]: https://git.sr.ht/~sircmpwn/scdoc


> this document is based on
> https: //git.sr.ht/~rjarry/aerc/tree/master/item/CONTRIBUTING.md
> Copyright (c) 2018-2019 Drew DeVault
> Copyright (c) 2021-2022 Robin Jarry
>
> under the MIT License
> Permission is hereby granted, free of charge, to any person obtaining a copy of
> this software and associated documentation files (the "Software"), to deal in
> the Software without restriction, including without limitation the rights to
> use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
> of the Software, and to permit persons to whom the Software is furnished to do
> so, subject to the following conditions:

> The above copyright notice and this permission notice shall be included in all
> copies or substantial portions of the Software.

> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
> IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
> FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
> AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
> LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
> OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
> SOFTWARE.
