# Sort the ioc file for better git diff
IOC=$(find *.ioc)
echo "Sorting $IOC for better git diffs"
sort $IOC --version-sort --output=$IOC 

FILES_TO_FORMAT=$(find Inc Src  -iname "*.h" -o -iname "*.c" -o -iname "*.cpp" -o -iname "*.hpp")
clang-format -i --verbose $FILES_TO_FORMAT

rm -rf Makefile