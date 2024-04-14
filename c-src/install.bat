@echo off
echo test
if not exist %userprofile%\\.pxec mkdir %userprofile%\\.pxec
copy px.exe %userprofile%\\.pxec\\px.exe
echo install complete
exit
