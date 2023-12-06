# SPDX-License-Identifier: Apache-2.0

import setuptools

setuptools.setup(
    name="nispor",
    version="1.2.15",
    author="Gris Ge",
    author_email="fge@redhat.com",
    description="Python binding of Nispor",
    long_description=(
        "Python binding of Nispor for kernel querying network state"
    ),
    url="https://github.com/nispor/nispor/",
    packages=setuptools.find_packages(),
    license="ASL2.0+",
    python_requires=">=3.6",
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: Apache Software License",
        "Operating System :: POSIX :: Linux",
    ],
)
