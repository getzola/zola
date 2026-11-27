- Feature Name: footnote-templates
- Start Date: 2026-11-02
- RFC PR: [zola/rfcs#0002](https://github.com/getzola/zola/pulls/0000)
## Summary
[summary]: #summary

Add the ability to use a Tera component to template the exact html generated for a footnote and/or the footnote reference.

## Motivation

I don’t like footnotes either in line at the point they were defined, they disrupt the reading flow, or at the bottom, which forces jumping up and down the page. On Desktop the screen is large enough, so that they can be to the side of the main content at the height of the footnote. To achieve this you need to use html anchors in the footnote and the reference with a custom attribute. By using a component and passing all the variables currently already used by the code for bottom footnotes it is possible to insert the required anchors and additional divs etc. to achieve my desired formatting.

This user used to define his footnotes directly as html, so they could style them more easily. In the future it would be one component definition and the normal usage of footnotes.

In #3100 one user would like to make the footnotes bigger and have more control over their styling. With a component using the footnote name, eg. [^note 1] it could print [note 1] in the text instead of [1] currently. Depending on how it is implemented it would also solve their wish for custom styling.

## Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

To use this feature you define a component with 4 parameters. (The exact naming is up for debate, I will use the current naming in the footnote code. One could possible also pass perma-link of them as an implicited parameter.) If no component is defined, the default formatting of zola takes place.

    perma-link: The permanent link to the current page, usefull if a user copies it.
    name: the content of the angle brackets, without the carret
    n: The count of all footnotes up to this point. This is what currently get’s rendered as the footnote.
    nr: How many references exists to this footnote or the number of the reference to this footnote, depending on, if it is a reference or the footnote itself.
    body: Passed implicitly only to the footnote and contains it’s content.

If you want to overwrite the rendering of a footnote reference to the name passed in brackets you need to create a new component like this:

{% component footnote_reference(perma-link, name, nr, n) %}
<sup class="footnote-reference" id="fr-{{name}}-{{nr}}">
    <a href="#fn-{{name}}">[{{name}}]</a>
</sup>
{% endcomponent %}

To also get the matching rendering in the footnote we need a second component for the footnote itself:

{% component footnote(perma-link, name, nr, n) %}
<sup class="footnote" id="fn-{{name}}">
    [{{name}}]
</sup>
{{body}}
{% endcomponent %}

If the component is wrong, the default Tera error would be emitted.

There would be no need to provide a migration guide, by default the current formatting would be used and only when a component is defined it would overwrite the normal process.

Sites currently using custom footnotes could switch over to the templated footnotes. It would also guarantee the expected html every time, in contrast to manual written html. I would therefor expect the usage of this feature to help maintain pages. If you change your mind of how they should look, it would now be a single place to change.

## Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

I have already implemented a draft of this RFC here, though it should be noted, that it currently doesn’t keep the output of everything before stable, so it fails some tests. This is because of the difference in output between bottom footnotes and inline notes and the [ ] behaviour.

The only other feature it interacts with is bottom_footnotes.

Currently if bottom_footnotes is false the emitted cmark footnotes output is passed along to the normal generator. With this change we would need to always handle them. Depending on if a component with a name of footnotes_reference is defined we either just call this component or use a format! path to get our old behaviour.

The same is done for footnotes themself, exepct we also need to condense the content of the footnote down to the string so we can pass it as the body.

## Drawbacks
[drawbacks]: #drawbacks

It would make the code for the footnotes more complicated, as now we have a branch between a defined component + calling tera and just using the format! call.

The site would probably be slower to render with Tera than with format!, however I don’t this impact will be noticeable.

## Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

An alternative would be to just not support this.

The other would be to build a full an more flexible scripting language into zola, this would probably go against all current goals as I understand them.

## Prior art
[prior-art]: #prior-art

I couldn’t really find nice prior art of this feature.

One Jekyll theme (kau.sh/blog/henry-jekyll-theme) just uses JS to implement sidenotes.

Jekyll itself doesn’t seem to bring this functionalty itself. It might be possible to do this with a custom plugin though.

In Hugo it sometimes is emulated with shortcodes.

Here a Hugo user is abusing the image subtitle feature to get the ability to modify the footnotes. abstractnonsense.xyz/microblog/2025-08-11-writing-collapsible-inline-footnotes-in-hugo-with-markdown/

Pelican doesn’t seem to bring a footnote implementation by default, however there is a plugin written in python that creates footnotes. Python is of course a lot more powerful and easy to change to fit the wishes of the user.

In short other SSG seem to use complete scripting languages to achieve similar result, be it at runtime or at build time. This Solution would just reuse the already familiar templating already in use for Zola themes.

## Unresolved questions
[unresolved-questions]: #unresolved-questions


Should we give the ability to emit a footnote twice, once at the position of the definition and once in the bottom_footnote footer?

How much state should be passed down into the component as implicit state?

Do we need html escaping of any of the passed variables.

If we already modify the emitted output for footnotes, should be always the same, wether bottom_footnotes is active or not?

Should these kind of discussion be posted here or in the github repo under the rfc folder?

## Future possibilities
[future-possibilities]: #future-possibilities

I don’t have much more Ideas in this direction currently.

Think about what the natural extension and evolution of your proposal would be and how it would affect Zola and project as a whole in a holistic way. Try to use this section as a tool to more fully consider all possible interactions with the project in your proposal.

This is also a good place to “dump ideas”, if they are out of scope for the RFC you are writing but otherwise related.

If you have tried and cannot think of any future possibilities, you may simply state that you cannot think of anything.

Note that having something written down in the future-possibilities section is not a reason to accept the current or a future RFC; such notes should be in the section on motivation or rationale in this or subsequent RFCs. The section merely provides additional information.
