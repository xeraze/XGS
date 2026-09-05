package com.gamepresence.gui;

import javafx.application.Application;
import javafx.application.Platform;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.*;
import javafx.scene.effect.*;
import javafx.scene.layout.*;
import javafx.scene.paint.Color;
import javafx.scene.paint.CycleMethod;
import javafx.scene.paint.LinearGradient;
import javafx.scene.paint.Stop;
import javafx.scene.shape.Rectangle;
import javafx.scene.text.Font;
import javafx.scene.text.FontWeight;
import javafx.scene.text.Text;
import javafx.stage.Stage;
import javafx.stage.StageStyle;

import java.io.*;
import java.nio.file.*;

public class Main extends Application {

    private static final String APP_NAME = "XGameStats";
    private static final String APP_VERSION = "0.1.0";
    private static final String CONFIG_DIR = System.getProperty("user.home") +
            File.separator + "AppData" + File.separator + "Local" +
            File.separator + "XGameStats";

    private static final Color BG_BASE = Color.rgb(12, 12, 18);
    private static final Color BG_CARD = Color.rgb(22, 22, 30);
    private static final Color BG_GLASS = Color.rgb(255, 255, 255, 0.05);
    private static final Color ACCENT = Color.rgb(99, 179, 237);
    private static final Color ACCENT_GLOW = Color.rgb(99, 179, 237, 0.4);
    private static final Color TEXT = Color.rgb(240, 240, 245);
    private static final Color TEXT_DIM = Color.rgb(120, 120, 135);
    private static final Color GREEN = Color.rgb(72, 199, 142);
    private static final Color RED = Color.rgb(239, 68, 68);
    private static final Color BORDER = Color.rgb(255, 255, 255, 0.08);

    private Process engineProcess;
    private TextArea logArea;
    private Label statusLabel;

    @Override
    public void start(Stage primaryStage) {
        showLoadingScreen(primaryStage);
    }

    private void showLoadingScreen(Stage primaryStage) {
        StackPane root = new StackPane();
        root.setBackground(new Background(new BackgroundFill(BG_BASE, null, null)));

        VBox loadBox = new VBox(20);
        loadBox.setAlignment(Pos.CENTER);

        Text title = new Text(APP_NAME);
        title.setFont(Font.font("Segoe UI", FontWeight.BOLD, 42));
        title.setFill(TEXT);
        title.setEffect(new Glow(0.3));

        ProgressIndicator spinner = new ProgressIndicator();
        spinner.setStyle("-fx-progress-color: #63B3ED;");
        spinner.setMaxSize(50, 50);

        Text status = new Text("Loading...");
        status.setFont(Font.font("Segoe UI", 14));
        status.setFill(TEXT_DIM);

        loadBox.getChildren().addAll(title, spinner, status);
        root.getChildren().add(loadBox);

        Scene scene = new Scene(root, 500, 350);
        primaryStage.initStyle(StageStyle.UNDECORATED);
        primaryStage.setScene(scene);
        primaryStage.centerOnScreen();
        primaryStage.show();

        new Thread(() -> {
            try { Thread.sleep(1500); } catch (InterruptedException ignored) {}
            Platform.runLater(() -> {
                primaryStage.close();
                showMainUI(primaryStage);
            });
        }).start();
    }

    private void showMainUI(Stage primaryStage) {
        primaryStage.setTitle(APP_NAME);

        StackPane root = new StackPane();
        root.setBackground(new Background(new BackgroundFill(BG_BASE, null, null)));

        Rectangle glassPane = new Rectangle(800, 550);
        glassPane.setFill(BG_GLASS);
        glassPane.setEffect(new GaussianBlur(40));
        glassPane.setX(-200);
        glassPane.setY(-100);

        VBox mainLayout = new VBox(0);
        mainLayout.setBackground(Background.EMPTY);

        mainLayout.getChildren().addAll(
                createHeader(primaryStage),
                createTabBar(),
                createContentArea(),
                createStatusBar()
        );

        root.getChildren().addAll(glassPane, mainLayout);

        Scene scene = new Scene(root, 800, 550);
        primaryStage.setScene(scene);
        primaryStage.centerOnScreen();
        primaryStage.show();
    }

    private HBox createHeader(Stage stage) {
        HBox header = new HBox();
        header.setAlignment(Pos.CENTER_LEFT);
        header.setPadding(new Insets(18, 24, 18, 24));
        header.setBackground(new Background(new BackgroundFill(
                Color.rgb(16, 16, 22, 0.9), null, null)));

        Region dragRegion = new Region();
        HBox.setHgrow(dragRegion, Priority.ALWAYS);
        dragRegion.setOnMousePressed(e -> {
            // drag support
        });

        VBox titleBox = new VBox(2);
        Text title = new Text(APP_NAME);
        title.setFont(Font.font("Segoe UI", FontWeight.BOLD, 20));
        title.setFill(ACCENT);
        title.setEffect(new Glow(0.2));

        Text subtitle = new Text("Turn any singleplayer game into Discord Rich Presence");
        subtitle.setFont(Font.font("Segoe UI", 11));
        subtitle.setFill(TEXT_DIM);

        titleBox.getChildren().addAll(title, subtitle);

        HBox controls = new HBox(10);
        controls.setAlignment(Pos.CENTER_RIGHT);

        Button startBtn = createGlowButton("Start", GREEN);
        startBtn.setOnAction(e -> startEngine());

        Button stopBtn = createGlowButton("Stop", RED);
        stopBtn.setOnAction(e -> stopEngine());

        Button closeBtn = createCircleButton("x", RED);
        closeBtn.setOnAction(e -> {
            stopEngine();
            Platform.exit();
        });

        controls.getChildren().addAll(startBtn, stopBtn, closeBtn);
        header.getChildren().addAll(titleBox, dragRegion, controls);

        return header;
    }

    private Button createGlowButton(String text, Color color) {
        Button btn = new Button(text);
        btn.setFont(Font.font("Segoe UI", FontWeight.BOLD, 12));
        btn.setTextFill(Color.WHITE);
        btn.setBackground(new Background(new BackgroundFill(color, new CornerRadii(6), null)));

        DropShadow shadow = new DropShadow();
        shadow.setColor(color.deriveColor(0, 1, 1, 0.5));
        shadow.setRadius(12);
        shadow.setSpread(0.2);
        btn.setEffect(shadow);

        btn.setOnMouseEntered(e -> {
            btn.setBackground(new Background(new BackgroundFill(
                    color.brighter(), new CornerRadii(6), null)));
            shadow.setRadius(18);
        });
        btn.setOnMouseExited(e -> {
            btn.setBackground(new Background(new BackgroundFill(
                    color, new CornerRadii(6), null)));
            shadow.setRadius(12);
        });

        btn.setPadding(new Insets(8, 20, 8, 20));
        btn.setCursor(javafx.scene.Cursor.HAND);
        btn.setStyle("-fx-background-color: transparent;");

        return btn;
    }

    private Button createCircleButton(String text, Color color) {
        Button btn = new Button(text);
        btn.setFont(Font.font("Segoe UI", FontWeight.BOLD, 14));
        btn.setTextFill(TEXT_DIM);
        btn.setBackground(Background.EMPTY);
        btn.setMinSize(32, 32);
        btn.setMaxSize(32, 32);

        btn.setOnMouseEntered(e -> btn.setTextFill(RED));
        btn.setOnMouseExited(e -> btn.setTextFill(TEXT_DIM));

        return btn;
    }

    private HBox createTabBar() {
        HBox tabBar = new HBox(0);
        tabBar.setPadding(new Insets(0, 24, 0, 24));
        tabBar.setBackground(new Background(new BackgroundFill(
                Color.rgb(18, 18, 24), null, null)));

        String[] tabs = {"Processes", "Configs", "Logs"};
        for (String tab : tabs) {
            Label label = new Label(tab);
            label.setFont(Font.font("Segoe UI", 12));
            label.setPadding(new Insets(12, 20, 12, 20));
            label.setTextFill(TEXT_DIM);
            label.setOnMouseEntered(e -> label.setTextFill(TEXT));
            label.setOnMouseExited(e -> label.setTextFill(TEXT_DIM));

            Separator sep = new Separator();
            sep.setOrientation(javafx.geometry.Orientation.VERTICAL);
            sep.setPadding(new Insets(8, 0, 8, 0));

            tabBar.getChildren().addAll(label, sep);
        }

        return tabBar;
    }

    private StackPane createContentArea() {
        StackPane content = new StackPane();
        content.setPadding(new Insets(20, 24, 20, 24));
        content.setBackground(new Background(new BackgroundFill(
                Color.rgb(14, 14, 20), new CornerRadii(12), null)));

        DropShadow cardShadow = new DropShadow();
        cardShadow.setColor(Color.rgb(0, 0, 0, 0.3));
        cardShadow.setRadius(20);
        cardShadow.setSpread(0.1);
        content.setEffect(cardShadow);

        VBox processPanel = createProcessPanel();
        content.getChildren().add(processPanel);

        return content;
    }

    private VBox createProcessPanel() {
        VBox panel = new VBox(15);
        panel.setPadding(new Insets(5));

        ListView<String> processList = new ListView<>();
        processList.setBackground(new Background(new BackgroundFill(
                Color.rgb(20, 20, 28), new CornerRadii(8), null)));
        processList.setPlaceholder(new Label("No games configured"));
        processList.getItems().addAll(loadProcessEntries());
        processList.setPrefHeight(300);

        HBox buttons = new HBox(10);
        buttons.setAlignment(Pos.CENTER_LEFT);

        Button addBtn = createAccentButton("Add Game");
        addBtn.setOnAction(e -> showAddDialog());

        Button removeBtn = createAccentButton("Remove");
        Button refreshBtn = createAccentButton("Refresh");

        buttons.getChildren().addAll(addBtn, removeBtn, refreshBtn);
        panel.getChildren().addAll(processList, buttons);

        return panel;
    }

    private Button createAccentButton(String text) {
        Button btn = new Button(text);
        btn.setFont(Font.font("Segoe UI", 12));
        btn.setTextFill(TEXT);
        btn.setBackground(new Background(new BackgroundFill(
                Color.rgb(45, 45, 55), new CornerRadii(6), null)));
        btn.setPadding(new Insets(8, 18, 8, 18));
        btn.setCursor(javafx.scene.Cursor.HAND);

        DropShadow shadow = new DropShadow();
        shadow.setColor(Color.rgb(0, 0, 0, 0.2));
        shadow.setRadius(8);
        btn.setEffect(shadow);

        btn.setOnMouseEntered(e -> {
            btn.setBackground(new Background(new BackgroundFill(
                    ACCENT.deriveColor(0, 0.6, 1, 1), new CornerRadii(6), null)));
        });
        btn.setOnMouseExited(e -> {
            btn.setBackground(new Background(new BackgroundFill(
                    Color.rgb(45, 45, 55), new CornerRadii(6), null)));
        });

        return btn;
    }

    private HBox createStatusBar() {
        HBox bar = new HBox();
        bar.setAlignment(Pos.CENTER_LEFT);
        bar.setPadding(new Insets(10, 24, 10, 24));
        bar.setBackground(new Background(new BackgroundFill(
                Color.rgb(14, 14, 20), null, null)));

        statusLabel = new Label("Idle");
        statusLabel.setFont(Font.font("Segoe UI", 11));
        statusLabel.setTextFill(TEXT_DIM);

        Region spacer = new Region();
        HBox.setHgrow(spacer, Priority.ALWAYS);

        Label version = new Label("v" + APP_VERSION);
        version.setFont(Font.font("Segoe UI", 10));
        version.setTextFill(TEXT_DIM);

        bar.getChildren().addAll(statusLabel, spacer, version);
        return bar;
    }

    private void showAddDialog() {
        Stage dialog = new Stage();
        dialog.initStyle(StageStyle.UNDECORATED);
        dialog.setTitle("Add Game");

        VBox root = new VBox(15);
        root.setPadding(new Insets(25));
        root.setBackground(new Background(new BackgroundFill(
                BG_CARD, new CornerRadii(12), null)));

        DropShadow shadow = new DropShadow();
        shadow.setColor(Color.rgb(0, 0, 0, 0.5));
        shadow.setRadius(30);
        root.setEffect(shadow);

        Text title = new Text("Add Game Configuration");
        title.setFont(Font.font("Segoe UI", FontWeight.BOLD, 16));
        title.setFill(TEXT);

        TextField processField = createStyledField("SFHR.exe");
        TextField appIdField = createStyledField("123456789012345");

        HBox buttons = new HBox(10);
        buttons.setAlignment(Pos.CENTER_RIGHT);

        Button cancelBtn = createAccentButton("Cancel");
        cancelBtn.setOnAction(e -> dialog.close());

        Button okBtn = createGlowButton("Add", ACCENT);
        okBtn.setOnAction(e -> {
            String process = processField.getText();
            String appId = appIdField.getText();
            if (!process.isEmpty() && !appId.isEmpty()) {
                saveConfig(process, appId);
                dialog.close();
            }
        });

        buttons.getChildren().addAll(cancelBtn, okBtn);

        root.getChildren().addAll(title, processField, appIdField, buttons);

        Scene scene = new Scene(root, 380, 250);
        scene.setFill(Color.TRANSPARENT);
        dialog.setScene(scene);
        dialog.initStyle(StageStyle.TRANSPARENT);
        dialog.centerOnScreen();
        dialog.show();
    }

    private TextField createStyledField(String prompt) {
        TextField field = new TextField();
        field.setPromptText(prompt);
        field.setFont(Font.font("Segoe UI", 13));
        field.setBackground(new Background(new BackgroundFill(
                Color.rgb(30, 30, 40), new CornerRadii(6), null)));
        field.setStyle("-fx-text-fill: #F0F0F5; -fx-prompt-text-fill: #787887;");
        field.setPadding(new Insets(10, 14, 10, 14));

        DropShadow glow = new DropShadow();
        glow.setColor(ACCENT_GLOW);
        glow.setRadius(0);
        field.focusedProperty().addListener((obs, old, val) -> {
            if (val) glow.setRadius(10);
            else glow.setRadius(0);
        });
        field.setEffect(glow);

        return field;
    }

    private void startEngine() {
        try {
            String exePath = findExe();
            if (exePath == null) {
                appendLog("[XGS] Engine not found. Run build_all.bat first.");
                statusLabel.setText("Error: engine not found");
                statusLabel.setTextFill(RED);
                return;
            }

            new File(CONFIG_DIR).mkdirs();

            ProcessBuilder pb = new ProcessBuilder(exePath);
            pb.directory(new File(CONFIG_DIR));
            pb.redirectErrorStream(true);

            engineProcess = pb.start();

            Thread reader = new Thread(() -> {
                try (BufferedReader r = new BufferedReader(
                        new InputStreamReader(engineProcess.getInputStream()))) {
                    String line;
                    while ((line = r.readLine()) != null) {
                        final String logLine = line;
                        Platform.runLater(() -> appendLog(logLine));
                    }
                } catch (IOException e) {
                    Platform.runLater(() -> appendLog("[ERROR] " + e.getMessage()));
                }
            });
            reader.setDaemon(true);
            reader.start();

            statusLabel.setText("Running");
            statusLabel.setTextFill(GREEN);
            appendLog("[XGS] Engine started");
        } catch (Exception e) {
            appendLog("[ERROR] " + e.getMessage());
            statusLabel.setText("Error");
            statusLabel.setTextFill(RED);
        }
    }

    private void stopEngine() {
        if (engineProcess != null && engineProcess.isAlive()) {
            engineProcess.destroy();
            statusLabel.setText("Stopped");
            statusLabel.setTextFill(RED);
        }
    }

    private String findExe() {
        String[] paths = {
            "xgs.exe",
            "core\\target\\release\\xgs.exe",
            "..\\core\\target\\release\\xgs.exe",
        };
        for (String p : paths) {
            if (new File(p).exists()) return new File(p).getAbsolutePath();
        }
        return null;
    }

    private void appendLog(String msg) {
        if (logArea != null) {
            logArea.appendText(msg + "\n");
        }
    }

    private String[] loadProcessEntries() {
        File dir = new File(CONFIG_DIR);
        if (!dir.exists()) return new String[0];
        File[] files = dir.listFiles((d, n) -> n.endsWith(".json"));
        if (files == null) return new String[0];
        String[] entries = new String[files.length];
        for (int i = 0; i < files.length; i++) {
            entries[i] = files[i].getName().replace(".json", "");
        }
        return entries;
    }

    private void saveConfig(String process, String appId) {
        try {
            File dir = new File(CONFIG_DIR);
            dir.mkdirs();

            String json = "{\n" +
                "  \"process_name\": \"" + process + "\",\n" +
                "  \"discord_app_id\": \"" + appId + "\",\n" +
                "  \"scan_type\": \"offsets\",\n" +
                "  \"requires_elevation\": false,\n" +
                "  \"pointers\": {},\n" +
                "  \"rpc_template\": {\n" +
                "    \"details\": \"Playing\",\n" +
                "    \"state\": \"In Game\"\n" +
                "  }\n" +
                "}";

            String name = process.replace(".exe", "").toLowerCase() + ".json";
            Files.write(new File(dir, name).toPath(), json.getBytes());
            appendLog("[XGS] Added: " + process);
        } catch (Exception e) {
            appendLog("[ERROR] " + e.getMessage());
        }
    }

    public static void main(String[] args) {
        launch(args);
    }
}
