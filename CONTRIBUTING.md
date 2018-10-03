# How to contribute

Your contributions are essential for making the irc crate the best piece of software it could
possibly be! We welcome contributions from programmers of all skill levels, and are happy to provide
mentorship for new contributors working on bug fixes. To make the onboarding process as painless as
possible for everyone, there are a few guidelines that we need contributors to follow.

## Getting Started

* Make sure you have a [GitHub account](https://github.com/signup/free).
* Fork the repository on GitHub.

## Finding Something to Do

There will almost certainly be issues open on the irc crate at all times. These are a good place to
start if you don't have anything pressing on your mind that you'd like to see. If you need or would
like mentoring instructions on any issue, please ping [@aatxe](https://github.com/aatxe), and he
will do his best to provide them in a timely fashion. If instead you have your own idea and would
like mentoring or feedback, you should open a new issue and mention such a desire.

## Making Changes

* Create a topic branch from where you want to base your work.
  * This is almost always the develop branch.
  * Only target stable or a maintainer's feature branch if you are certain your fix must be on that
    branch.
  * To quickly create a topic branch based on develop, run `git checkout -b
    fix/develop/my_contribution develop`. Please avoid working directly on the
    `stable` branch, and keep direct work on `develop` minor.
* Make commits of logical and atomic units.
* Check for unnecessary whitespace with `git diff --check` before committing.
* Make sure you have added the necessary tests for your changes.
* Run _all_ the tests to assure nothing else was accidentally broken using `cargo test`. You can
  also run `cargo test --no-default-features`, but this is generally not relevant and will be
  handled by our continuous integration tools.
  * Some tests related to configurations will fail if you don't run the `mktestconfig.sh` script.

## Submitting Changes

* Push your changes to a topic branch in your fork of the repository.
* Submit a pull request to the `aatxe/irc` repository.
* Await maintainer feedback and continuous integration results.
  * We make an effort to triage quickly, but patience is appreciated!

## Additional Resources

* [Code of Conduct](https://github.com/aatxe/irc/blob/develop/CODE_OF_CONDUCT.md)
* [the irc crate docs](https://docs.rs/irc)
* #rust-irc IRC channel on irc.mozilla.org

## Credits

These contributing guidelines were inspired by the
[Puppet contributor guidelines](https://github.com/puppetlabs/puppet/blob/master/CONTRIBUTING.md).
