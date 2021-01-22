cd doc-gen
cargo run
cd ..

rm -Recurse ./book/src/api
rm -Recurse ./docs

cp -r ./doc-gen/output/api ./book/src
cp ./book/src/SUMMARY_TEMPLATE.md ./book/src/SUMMARY.md
Get-Content -Path ./doc-gen/output/SUMMARY.md | Add-Content -Path ./book/src/SUMMARY.md

cd book
mdbook build
cd ..
mv docs/print.html docs/index.html
