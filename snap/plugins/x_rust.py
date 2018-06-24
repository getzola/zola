# -*- Mode:Python; indent-tabs-mode:nil; tab-width:4 -*-
#
# Copyright (C) 2016-2017 Marius Gripsgard (mariogrip@ubuntu.com)
# Copyright (C) 2016-2017 Canonical Ltd
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License version 3 as
# published by the Free Software Foundation.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.

"""This rust plugin is useful for building rust based parts.

Rust uses cargo to drive the build.

This plugin uses the common plugin keywords as well as those for "sources".
For more information check the 'plugins' topic for the former and the
'sources' topic for the latter.

Additionally, this plugin uses the following plugin-specific keywords:

    - rust-channel
      (string)
      select rust channel (stable, beta, nightly)
    - rust-revision
      (string)
      select rust version
    - rust-features
      (list of strings)
      Features used to build optional dependencies
"""

import collections
import os
import shutil
from contextlib import suppress

import snapcraft
from snapcraft import sources
from snapcraft import shell_utils
from snapcraft.internal import errors

_RUSTUP = 'https://static.rust-lang.org/rustup.sh'


class RustPlugin(snapcraft.BasePlugin):
    @classmethod
    def schema(cls):
        schema = super().schema()
        schema['properties']['rust-channel'] = {
            'type': 'string',
        }
        schema['properties']['rust-revision'] = {
            'type': 'string',
        }
        schema['properties']['rust-features'] = {
            'type': 'array',
            'minitems': 1,
            'uniqueItems': True,
            'items': {
                'type': 'string',
            },
            'default': []
        }

        return schema

    @classmethod
    def get_pull_properties(cls):
        return ['rust-revision', 'rust-channel']

    @classmethod
    def get_build_properties(cls):
        return ['rust-features']

    def __init__(self, name, options, project):
        super().__init__(name, options, project)
        self.build_packages.extend([
            'gcc',
            'git',
            'curl',
            'file',
        ])
        self._rustpath = os.path.join(self.partdir, "rust")
        self._rustc = os.path.join(self._rustpath, "bin", "rustc")
        self._rustdoc = os.path.join(self._rustpath, "bin", "rustdoc")
        self._cargo = os.path.join(self._rustpath, "bin", "cargo")
        self._cargo_dir = os.path.join(self.builddir, '.cargo')
        self._rustlib = os.path.join(self._rustpath, "lib")
        self._rustup_get = sources.Script(_RUSTUP, self._rustpath)
        self._rustup = os.path.join(self._rustpath, "rustup.sh")
        self._manifest = collections.OrderedDict()

    def _test(self):
        cmd = [self._cargo, 'test',
               '-j{}'.format(self.parallel_build_count)]
        if self.options.rust_features:
            cmd.append("--features")
            cmd.append(' '.join(self.options.rust_features))
        self.run(cmd, env=self._build_env())

    def build(self):
        super().build()

        self._write_cross_compile_config()

        self._test()

        cmd = [self._cargo, 'install',
               '-j{}'.format(self.parallel_build_count),
               '--root', self.installdir,
               '--path', self.builddir]
        if self.options.rust_features:
            cmd.append("--features")
            cmd.append(' '.join(self.options.rust_features))
        self.run(cmd, env=self._build_env())
        self._record_manifest()

    def _write_cross_compile_config(self):
        if not self.project.is_cross_compiling:
            return

        # Cf. http://doc.crates.io/config.html
        os.makedirs(self._cargo_dir, exist_ok=True)
        with open(os.path.join(self._cargo_dir, 'config'), 'w') as f:
            f.write('''
                [build]
                target = "{}"

                [target.{}]
                linker = "{}"
                '''.format(self._target, self._target,
                           '{}-gcc'.format(self.project.arch_triplet)))

    def _record_manifest(self):
        self._manifest['rustup-version'] = self.run_output(
            [self._rustup, '--version'])
        self._manifest['rustc-version'] = self.run_output(
            [self._rustc, '--version'])
        self._manifest['cargo-version'] = self.run_output(
            [self._cargo, '--version'])
        with suppress(FileNotFoundError, IsADirectoryError):
            with open(os.path.join(self.builddir, 'Cargo.lock')) as lock_file:
                self._manifest['cargo-lock-contents'] = lock_file.read()

    def get_manifest(self):
        return self._manifest

    def enable_cross_compilation(self):
        # Cf. rustc --print target-list
        targets = {
            'armhf': 'armv7-{}-{}eabihf',
            'arm64': 'aarch64-{}-{}',
            'i386': 'i686-{}-{}',
            'amd64': 'x86_64-{}-{}',
            'ppc64el': 'powerpc64le-{}-{}',
        }
        fmt = targets.get(self.project.deb_arch)
        if not fmt:
            raise NotImplementedError(
                '{!r} is not supported as a target architecture when '
                'cross-compiling with the rust plugin'.format(
                    self.project.deb_arch))
        self._target = fmt.format('unknown-linux', 'gnu')

    def _build_env(self):
        env = os.environ.copy()
        env.update({"RUSTC": self._rustc,
                    "RUSTDOC": self._rustdoc,
                    "RUST_PATH": self._rustlib,
                    'RUSTFLAGS': self._rustflags()})
        return env

    def _rustflags(self):
        ldflags = shell_utils.getenv('LDFLAGS')
        rustldflags = ''
        flags = {flag for flag in ldflags.split(' ') if flag}
        for flag in flags:
            rustldflags += '-C link-arg={} '.format(flag)
        return rustldflags.strip()

    def pull(self):
        super().pull()
        self._fetch_rust()
        self._fetch_deps()

    def clean_pull(self):
        super().clean_pull()

        with suppress(FileNotFoundError):
            shutil.rmtree(self._rustpath)

    def clean_build(self):
        super().clean_build()

        with suppress(FileNotFoundError):
            shutil.rmtree(self._cargo_dir)

    def _fetch_rust(self):
        options = []

        if self.options.rust_revision:
            options.append('--revision={}'.format(self.options.rust_revision))

        if self.options.rust_channel:
            if self.options.rust_channel in ['stable', 'beta', 'nightly']:
                options.append(
                    '--channel={}'.format(self.options.rust_channel))
            else:
                raise errors.SnapcraftEnvironmentError(
                    '{} is not a valid rust channel'.format(
                        self.options.rust_channel))
        os.makedirs(self._rustpath, exist_ok=True)
        self._rustup_get.download()
        cmd = [self._rustup,
               '--prefix={}'.format(self._rustpath),
               '--disable-sudo', '--save'] + options
        if self.project.is_cross_compiling:
            cmd.append('--with-target={}'.format(self._target))
        self.run(cmd)

    def _fetch_deps(self):
        if self.options.source_subdir:
            sourcedir = os.path.join(self.sourcedir,
                                     self.options.source_subdir)
        else:
            sourcedir = self.sourcedir

        self.run([self._cargo, 'fetch',
                  '--manifest-path',
                  os.path.join(sourcedir, 'Cargo.toml')],
                 env=self._build_env())
