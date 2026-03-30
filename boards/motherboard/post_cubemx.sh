echo "Removing unused CubeMX files"
rm -rfv Src/app_freertos.c
rm -rfv Inc/app_freertos.h

IOC=$(find *.ioc)
echo "Sorting $IOC for better git diffs"
sort $IOC --version-sort --output=$IOC 

FILES_TO_FORMAT=$(find Inc Src  -iname "*.h" -o -iname "*.c" -o -iname "*.cpp" -o -iname "*.hpp")
clang-format -i --verbose $FILES_TO_FORMAT
clang-format -i $FILES_TO_FORMAT # clang-format has some idempotency issues. run it twice
