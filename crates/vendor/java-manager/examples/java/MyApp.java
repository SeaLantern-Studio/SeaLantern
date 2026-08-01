import java.util.Scanner;

public class MyApp {
    public static void main(String[] args) {
        System.out.println("Hello from myapp.jar");
        System.out.println("Arguments received:");
        for (int i = 0; i < args.length; i++) {
            System.out.println("  args[" + i + "] = " + args[i]);
        }

        System.out.println("Input some text: ");
        Scanner scanner = new Scanner(System.in);
        String input = scanner.nextLine();
        System.out.println("You entered: " + input);

        Runtime runtime = Runtime.getRuntime();
        long maxMemory = runtime.maxMemory();
        long totalMemory = runtime.totalMemory();
        long freeMemory = runtime.freeMemory();

        System.out.println("Max memory (-Xmx): " + (maxMemory / 1024 / 1024) + " MB");
        System.out.println("Total memory (currently allocated): " + (totalMemory / 1024 / 1024) + " MB");
        System.out.println("Free memory: " + (freeMemory / 1024 / 1024) + " MB");

        System.err.println("Error: Something went wrong in MyApp");
    }
}