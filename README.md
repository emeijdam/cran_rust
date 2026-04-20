https://blog.sellorm.com/2019/03/30/build-your-own-cran-like-repo/

https://cran.r-project.org/doc/manuals/R-admin.html#Setting-up-a-package-repository


rsync -av -f '- /*/*/' --progress cran.r-project.org::CRAN/ ./test
rsync -av -f '- /*/*/' --progress cran.r-project.org::CRAN/bin/windows/base/ ./test/bin/windows/
rsync -av -f '- /*/*/' --progress cran.r-project.org::CRAN/bin/windows/base/ ./test/bin/macosx/
rsync -av -f '- /*/*/' --progress cran.r-project.org::CRAN/bin/windows/base/ ./test/bin/linux/

rsync -av -f '- /*/*/' --progress cran.r-project.org::CRAN/bin/windows/base/ ./test/bin/linux/


## heavy --dry-run
rsync -av -f '- /*/*/' --progress cran.r-project.org::CRAN/web/ ./test/ 

rsync -av -f '- /*/*/' --progress cran.r-project.org::CRAN/src ./test/ --dry-run


 rsync -av -f '- /*/*/' --progress cran.r-project.org::CRAN/web/packages ./test/web

  rsync -av -f '- /*/*/' --progress cran.r-project.org::CRAN/src/ ./test/

works
rsync -av -f '- /*/*/' --progress cran.r-project.org::CRAN/web/packages ./test/web

rsync -av -f '- /*/*/' --progress cran.r-project.org::CRAN/web/packages/ggplot2 ./test/web/packages


rsync -av -f '- /*/*/' --progress cran.r-project.org::CRAN/web/packages ./test/web


# VERSIOM - INFO
rsync -av -f '- /*/*/' --progress cran.r-project.org::CRAN/src/base ./test/src

# VERSIOM - PACKAGES
# https://cran.r-project.org/src/contrib/PACKAGES

rsync -av -f '/*' --progress cran.r-project.org::CRAN/src/contrib ./test/src

rsync -av --exclude='/'  --exclude='*.tar.gz' --progress cran.r-project.org::CRAN/src/contrib ./test/src


# gt 

gt,1.3.0

https://cloud.r-project.org/bin/windows/contrib/4.5/gt_1.3.0.zip


https://cloud.r-project.org/bin/windows/contrib/4.5/


rsync -av --exclude='/'  --exclude='*.tar.gz' --progress cran.r-project.org::CRAN/bin/windows/contrib/4.5 ./test/bin/windows/contrib/



# SOURCES
https://cran.r-project.org/src/base/VERSION-INFO.dcf
 
https://cran.r-project.org/src/contrib/Meta/archive.rds

https://cran.r-project.org/web/packages/packages.rds

https://www.googleapis.com/download/storage/v1/b/osv-vulnerabilities/o/CRAN%2Fall.zip

https://www.googleapis.com/download/storage/v1/b/osv-vulnerabilities/o/CRAN%2Fmodified_id.csv?generation=1775720516055232&alt=media

https://cran.r-project.org/bin/windows/base/old/


cargo run -- dcf ./test/VERSION-INFO.dcf data.json
cargo run -- rds ./data/packages.rds packages.csv
cargo run -- rds ./data/archive.rds archive.csv

cargo build --target x86_64-unknown-linux-gnu --release


git checkout --orphan temp_branch
git add -A 
git commit -m "Initial commit" 
git branch -D master 
it branch --all   
git branch -m master 