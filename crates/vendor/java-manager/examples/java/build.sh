#!/bin/bash

echo "Building Java code..."
echo "Compiling App.java..."
javac App.java && jar cfe app.jar App App.class
echo "Compiling ErrorApp.java..."
javac ErrorApp.java && jar cfe error.jar ErrorApp ErrorApp.class
echo "Compiling MyApp.java..."
javac MyApp.java && jar cfe myapp.jar MyApp MyApp.class

echo "Java code built successfully!"
echo "Moving JAR files to project directory..."
mv app.jar error.jar myapp.jar ../

echo "Cleaning up..."
rm *.class
echo "Done!"
