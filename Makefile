build:
	gcc main.c -o px
install:
	cp ./px /usr/local/bin/px && rm ./px
install-windows:
	start "install.bat"
