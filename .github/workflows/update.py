import os
import re


def extract(filepath: str, pattern: str) -> str:
    with open(filepath, 'r') as file:
        for line in file:
            m = re.match(pattern, line)
            if m:
                return m.group(1)

def replace_file(filepath: str, pattern:str, replacement: str):
    with open(filepath, 'r') as file:
        file_contents = file.read()
        replaced_text = re.sub(pattern, replacement, file_contents)

    with open(filepath, 'w') as file:
        file.write(replaced_text)

rust_version = extract('rust-toolchain', r'(\d+\.\d+\.\d+)')
print(rust_version)

# workflows
replace_filepath = '.github/workflows/test.yml'
replace_pattern = r'dtolnay/rust-toolchain@\d+\.\d+\.\d+'
replace_text = r'dtolnay/rust-toolchain@{0}'.format(rust_version)
replace_file(replace_filepath, replace_pattern, replace_text)

replace_filepath = '.github/workflows/test-slim.yml'
replace_pattern = r'dtolnay/rust-toolchain@\d+\.\d+\.\d+'
replace_text = r'dtolnay/rust-toolchain@{0}'.format(rust_version)
replace_file(replace_filepath, replace_pattern, replace_text)

# dockerfiles
chef_version = extract('Dockerfile.build', r'FROM lukemathwalker\/cargo-chef:(\d+\.\d+\.\d+)-rust-\d+\.\d+\.\d+-slim-buster AS chef')
print(chef_version)

replace_filepath = 'Dockerfile.build'
replace_pattern = r'lukemathwalker/cargo-chef:\d+\.\d+\.\d+-rust-\d+\.\d+\.\d+-slim-buster'
replace_text = r'lukemathwalker/cargo-chef:{0}-rust-{1}-slim-buster'.format(chef_version, rust_version)
replace_file(replace_filepath, replace_pattern, replace_text)

# github outputs
with open(os.environ.get("GITHUB_OUTPUT", "github_output.txt"), "a") as file:
    file.write('rust_version={0}'.format(rust_version))