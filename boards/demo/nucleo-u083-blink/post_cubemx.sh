# Sort the ioc file for better git diff
IOC=$(find *.ioc)
echo "Sorting $IOC for better git diffs"
sort $IOC --version-sort --output=$IOC 

FILES_TO_FORMAT=$(find .  -path "./Drivers" -prune -false -o -iname "*.h" -or -iname "*.c" -or -iname "*.cpp" -or -iname "*.hpp")
clang-format -i --verbose $FILES_TO_FORMAT

rm -rf Makefile
