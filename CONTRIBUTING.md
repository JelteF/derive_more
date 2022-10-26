Contribution Guide
==================

🎈 Thanks for your help improving the project! We are so happy to have you!

__No contribution is too small and all contributions are valued.__




## Pull Requests

[Pull Requests][PR] are the way concrete changes are made to the code, documentation, and dependencies in the `derive_more` repository.

Even tiny [PR]s (e.g., fixing a typo in API documentation) are greatly appreciated. Before making a large change, however, it's usually a good idea to first open an [issue] describing the change to solicit feedback and guidance. This will increase the likelihood of the [PR] getting merged.

Any of the guidelines described below may be ignored if it makes sense in the specific case, but following them should be the default.


### Breaking changes

Try to avoid introducing breaking changes in [PR]s.

If a new behaviour makes sense, that is different from the current behavior, then this new behaviour should only be enabled via an attribute. Of course, if the old behaviour doesn't make sense for any reasonable person to rely on, it's possible to ignore this guideline, but such a decision should be made actively and argued for.


### Documentation

Documentation is contained in the `impl/doc/*.md` files and [README].

Documentation should be up-to-date with any [PR] changes visible for library end-users.

#### Changelog

The same way, any [PR] changes visible for library end-users should be mentioned in the [CHANGELOG] file.

Consider to mention a [PR] number (and [issue], if possible) in the added [CHANGELOG] entries.


### Tests

If the change being proposed alters code (as opposed to only documentation, for example), it's either adding new functionality or is fixing existing, broken functionality. In both of these cases, the [PR] should include one or more tests to ensure that `derive_more` won't regress in the future.

There are multiple ways to write tests: integration tests, documentation tests and unit tests.

#### Integration tests

[Integration tests][3] are contained in the `tests/` directory of the repository.

The best strategy for writing a new integration test is to look at existing integration tests in the crate and follow the style.

#### Documentation tests

These are the [code examples][1] in the `impl/doc/*.md` files and [README].

Writing documentation tests is needed for better illustration of the added/altered capabilities for end-users of the crate.

#### Unit tests

[Unit tests][2] don't have much sense when it comes to macro testing. That's why they are rare beasts in the code of this repository. However, occasionally, they're very useful for testing some complicated properties locally (like correctness of syntax parsing, for example).


### Review

To get merged, any [PR] should be reviewed and approved by at least one of the active project maintainers, except the [PR] submitter, of course.

Furthermore, __any `derive_more` community member is welcome to review any [PR].__

All `derive_more` contributors who choose to review and provide feedback on [Pull Requests][PR] have a responsibility to both the project and the individual making the contribution. Reviews and feedback must be helpful, insightful, and geared towards improving the contribution as opposed to simply blocking it. If there are reasons why you feel the [PR] should not land, explain what those are. Do not expect to be able to block a [Pull Request][PR] from advancing simply because you say "No" without giving an explanation. Be open to having your mind changed. Be open to working with the contributor to make the [Pull Request][PR] better.

When reviewing a [Pull Request][PR], the primary goals are for the codebase to improve and for the person submitting the request to succeed. Even if a [Pull Request][PR] doesn't land, the submitters should come away from the experience feeling like their effort was not wasted or unappreciated.

#### Abandoned or stalled [Pull Requests][PR]

If a [Pull Request][PR] appears to be abandoned or stalled, it's polite to first check with the contributor to see if they intend to continue the work before checking if they would mind if you took it over (especially if it just has nits left). When doing so, it's courteous to give the original contributor credit for the work they started (by preserving their name and email address with `Co-authored-by:` meta-data tag in the commit).


### Merging

All [PR]s are [squash]-merged to preserve the main history linear and meaningful.

#### Commit message for a [squash] merge

Commit message for a [squash] merge of the [PR] should mention its number and the number of the relevant [issue] (if it has the one). The commit body should contain the [PR] description. All of this is done automatically by GitHub.




## Releasing

To produce a new release of the `derive_more` crate, perform the following steps:

1. Complete the existing [CHANGELOG] or fill up a new one for the new version.
2. Update [README] installation instructions with the new version.
3. Run `cargo release patch --workspace` (or `minor`/`major`).
4. Wait for the CI pipeline to complete successfully, and the [GitHub release] being created.




[`Cargo.toml`]: Cargo.toml
[CHANGELOG]: CHANGELOG.md
[GitHub release]: /../../releases
[issue]: /../../issues
[PR]: /../../pulls
[README]: README.md#installation
[squash]: https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/about-pull-request-merges#squash-and-merge-your-commits

[1]: https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html
[2]: https://doc.rust-lang.org/book/ch11-03-test-organization.html#unit-tests
[3]: https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests
