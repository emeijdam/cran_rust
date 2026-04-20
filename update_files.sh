#!/bin/bash

# Target directory
#TARGET="/opt/cran_data"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        TARGET="/opt/cran_data"
else
    TARGET="./cran_data"
fi

TARGET="./cran_data"
mkdir -p $TARGET
cd $TARGET

# Sync CRAN (Using Rsync Protocol)
rsync -rtlzP rsync://cran.r-project.org/CRAN/src/contrib/Meta/archive.rds .
rsync -rtlzP rsync://cran.r-project.org/CRAN/web/packages/packages.rds .
rsync -rtlzP rsync://cran.r-project.org/CRAN/src/base/VERSION-INFO.dcf.

# Sync OSV (Using Time-Conditional Downloads)
wget -N "https://storage.googleapis.com/osv-vulnerabilities/CRAN/all.zip"
wget -N "https://storage.googleapis.com/osv-vulnerabilities/CRAN/modified_id.csv"