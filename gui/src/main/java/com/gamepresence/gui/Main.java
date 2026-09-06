package com.gamepresence.gui;

import javafx.application.Application;
import javafx.application.Platform;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.*;
import javafx.scene.layout.*;
import javafx.scene.paint.Color;
import javafx.scene.text.Font;
import javafx.scene.text.FontWeight;
import javafx.scene.text.Text;
import javafx.stage.Screen;
import javafx.stage.Stage;
import javafx.stage.StageStyle;

import java.awt.AWTException;
import java.awt.BasicStroke;
import java.awt.FontMetrics;
import java.awt.Graphics2D;
import java.awt.MenuItem;
import java.awt.PopupMenu;
import java.awt.RenderingHints;
import java.awt.SystemTray;
import java.awt.TrayIcon;
import java.awt.event.MouseAdapter;
import java.awt.event.MouseEvent;
import java.awt.image.BufferedImage;
import java.io.*;
import java.nio.file.*;

public class Main extends Application {

    private static final String APP_NAME    = "XGameStats";
    private static final String APP_VERSION = "0.1.0";
    private static final String CONFIG_DIR  = System.getProperty("user.home")
            + File.separator + "AppData" + File.separator + "Local"
            + File.separator + "XGameStats";

    private static final Color ACCENT = Color.rgb(99, 179, 237);
    private static final Color TEXT   = Color.rgb(240, 240, 245);
    private static final Color DIM    = Color.rgb(120, 120, 135);
    private static final Color GREEN  = Color.rgb(72,  199, 142);
    private static final Color RED    = Color.rgb(239, 68,  68);

    private Process          engineProcess;
    private ListView<String> processList;
    private TextArea         logArea;
    private Label            statusLabel;
    private StackPane        contentPane;
    private HBox             tabBar;

    private TrayIcon trayIcon;
    private Stage    primaryStage;

    private double dragOffsetX, dragOffsetY;

    @Override
    public void start(Stage stage) {
        this.primaryStage = stage;
        Platform.setImplicitExit(false);
        installTray(stage);
        showSplash(stage);
    }

    private void showSplash(Stage stage) {
        StackPane root = new StackPane();
        root.setStyle("-fx-background-color: #0C0C12;");

        VBox box = new VBox(20);
        box.setAlignment(Pos.CENTER);

        Text t = new Text(APP_NAME);
        t.setFont(Font.font("Segoe UI", FontWeight.BOLD, 48));
        t.setFill(TEXT);

        Text v = new Text("v" + APP_VERSION);
        v.setFont(Font.font("Segoe UI", 13));
        v.setFill(DIM);

        ProgressIndicator sp = new ProgressIndicator();
        sp.setStyle("-fx-progress-color: #63B3ED;");
        sp.setMaxSize(40, 40);

        box.getChildren().addAll(t, v, sp);
        root.getChildren().add(box);

        Scene s = new Scene(root, 480, 320);
        stage.initStyle(StageStyle.UNDECORATED);
        stage.setScene(s);
        stage.show();
        centerOnScreen(stage);

        new Thread(() -> {
            try { Thread.sleep(1800); } catch (InterruptedException e) { Thread.currentThread().interrupt(); }
            Platform.runLater(() -> { stage.close(); showMain(stage); });
        }).start();
    }

    private void showMain(Stage stage) {
        BorderPane root = new BorderPane();
        root.setStyle("-fx-background-color: #0C0C12;");

        HBox header = createHeader(stage);
        root.setTop(header);
        root.setCenter(createContent());
        root.setBottom(createStatusBar());

        Scene s = new Scene(root, 800, 550);
        stage.setScene(s);
        stage.show();
        centerOnScreen(stage);

        header.setOnMousePressed(e -> {
            dragOffsetX = e.getScreenX() - stage.getX();
            dragOffsetY = e.getScreenY() - stage.getY();
        });
        header.setOnMouseDragged(e -> {
            stage.setX(e.getScreenX() - dragOffsetX);
            stage.setY(e.getScreenY() - dragOffsetY);
        });

        stage.setOnCloseRequest(e -> {
            e.consume();
            minimizeToTray(stage);
        });
    }

    private HBox createHeader(Stage stage) {
        HBox h = new HBox();
        h.setAlignment(Pos.CENTER_LEFT);
        h.setPadding(new Insets(12, 24, 12, 24));
        h.setStyle("-fx-background-color: #101016;");

        VBox tb = new VBox(2);
        Text t = new Text(APP_NAME);
        t.setFont(Font.font("Segoe UI", FontWeight.BOLD, 20));
        t.setFill(ACCENT);
        Text sub = new Text("Turn any singleplayer game into Discord Rich Presence");
        sub.setFont(Font.font("Segoe UI", 11));
        sub.setFill(DIM);
        tb.getChildren().addAll(t, sub);

        Region spacer = new Region();
        HBox.setHgrow(spacer, Priority.ALWAYS);

        Button btnStart = coloredBtn("Start", GREEN);
        btnStart.setOnAction(e -> startEngine());
        btnStart.setFocusTraversable(false);

        Button btnStop = coloredBtn("Stop", RED);
        btnStop.setOnAction(e -> stopEngine());
        btnStop.setFocusTraversable(false);

        Button btnMin = windowBtn("\u2212");
        btnMin.setOnAction(e -> minimizeToTray(stage));

        Button btnClose = windowBtn("\u00d7");
        btnClose.setOnMouseEntered(ev -> btnClose.setTextFill(RED));
        btnClose.setOnMouseExited(ev  -> btnClose.setTextFill(DIM));
        btnClose.setOnAction(e -> minimizeToTray(stage));

        HBox btns = new HBox(8);
        btns.setAlignment(Pos.CENTER_RIGHT);
        btns.getChildren().addAll(btnStart, btnStop, btnMin, btnClose);
        btns.setOnMousePressed(javafx.event.Event::consume);
        btns.setOnMouseDragged(javafx.event.Event::consume);

        h.getChildren().addAll(tb, spacer, btns);
        return h;
    }

    private void installTray(Stage stage) {
        if (!SystemTray.isSupported()) return;
        SystemTray tray = SystemTray.getSystemTray();

        PopupMenu menu = new PopupMenu();

        MenuItem itemShow  = new MenuItem("\u041e\u0442\u043a\u0440\u044b\u0442\u044c XGameStats");
        itemShow.addActionListener(e -> Platform.runLater(() -> showFromTray(stage)));

        MenuItem itemStart = new MenuItem("\u0417\u0430\u043f\u0443\u0441\u0442\u0438\u0442\u044c \u0434\u0432\u0438\u0436\u043e\u043a");
        itemStart.addActionListener(e -> Platform.runLater(this::startEngine));

        MenuItem itemStop  = new MenuItem("\u041e\u0441\u0442\u0430\u043d\u043e\u0432\u0438\u0442\u044c \u0434\u0432\u0438\u0436\u043e\u043a");
        itemStop.addActionListener(e -> Platform.runLater(this::stopEngine));

        MenuItem itemExit  = new MenuItem("\u0412\u044b\u0439\u0442\u0438");
        itemExit.addActionListener(e -> Platform.runLater(this::exitApp));

        menu.add(itemShow);
        menu.addSeparator();
        menu.add(itemStart);
        menu.add(itemStop);
        menu.addSeparator();
        menu.add(itemExit);

        trayIcon = new TrayIcon(buildTrayImage(), APP_NAME + " v" + APP_VERSION, menu);
        trayIcon.setImageAutoSize(true);
        trayIcon.addMouseListener(new MouseAdapter() {
            @Override public void mouseClicked(MouseEvent e) {
                if (e.getClickCount() >= 2) Platform.runLater(() -> showFromTray(stage));
            }
        });

        try { tray.add(trayIcon); } catch (AWTException ex) {
            System.err.println("[XGS] Tray init failed: " + ex.getMessage());
        }
    }

    private void minimizeToTray(Stage stage) {
        stage.hide();
        if (trayIcon != null) {
            trayIcon.displayMessage(APP_NAME,
                "\u0421\u0432\u0451\u0440\u043d\u0443\u0442\u043e \u0432 \u0442\u0440\u0435\u0439. \u0414\u0432\u043e\u0439\u043d\u043e\u0439 \u043a\u043b\u0438\u043a — \u043e\u0442\u043a\u0440\u044b\u0442\u044c.",
                TrayIcon.MessageType.INFO);
        }
    }

    private void showFromTray(Stage stage) {
        stage.show();
        stage.toFront();
    }

    private void exitApp() {
        stopEngine();
        if (trayIcon != null) SystemTray.getSystemTray().remove(trayIcon);
        Platform.exit();
        System.exit(0);
    }

    private java.awt.Image buildTrayImage() {
        int sz = 64;
        BufferedImage img = new BufferedImage(sz, sz, BufferedImage.TYPE_INT_ARGB);
        Graphics2D g = img.createGraphics();
        g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
        g.setColor(new java.awt.Color(12, 12, 24));
        g.fillOval(2, 2, sz - 4, sz - 4);
        g.setColor(new java.awt.Color(99, 179, 237));
        g.setStroke(new BasicStroke(3f));
        g.drawOval(4, 4, sz - 8, sz - 8);
        g.setColor(new java.awt.Color(240, 240, 245));
        g.setFont(new java.awt.Font("Segoe UI", java.awt.Font.BOLD, 34));
        FontMetrics fm = g.getFontMetrics();
        String ltr = "X";
        g.drawString(ltr, (sz - fm.stringWidth(ltr)) / 2, (sz - fm.getHeight()) / 2 + fm.getAscent());
        g.dispose();
        return img;
    }

    private Button windowBtn(String symbol) {
        Button b = new Button(symbol);
        b.setFont(Font.font("Segoe UI", FontWeight.BOLD, 14));
        b.setTextFill(DIM);
        b.setBackground(Background.EMPTY);
        b.setMinSize(32, 32);
        b.setMaxSize(32, 32);
        b.setFocusTraversable(false);
        b.setOnMouseEntered(e -> b.setTextFill(TEXT));
        b.setOnMouseExited(e  -> b.setTextFill(DIM));
        return b;
    }

    private Button coloredBtn(String text, Color color) {
        Button b = new Button(text);
        b.setFont(Font.font("Segoe UI", FontWeight.BOLD, 12));
        b.setTextFill(Color.WHITE);
        b.setStyle(colorStyle(color, 1.0));
        b.setOnMouseEntered(e -> b.setStyle(colorStyle(color, 1.1)));
        b.setOnMouseExited(e  -> b.setStyle(colorStyle(color, 1.0)));
        return b;
    }

    private String colorStyle(Color c, double f) {
        int r  = Math.min(255, (int)(c.getRed()   * 255 * f));
        int g  = Math.min(255, (int)(c.getGreen() * 255 * f));
        int bl = Math.min(255, (int)(c.getBlue()  * 255 * f));
        return "-fx-background-color: rgb(" + r + "," + g + "," + bl + "); "
             + "-fx-background-radius: 6; -fx-cursor: hand; -fx-padding: 8 20;";
    }

    private Button accentBtn(String text) {
        Button b = new Button(text);
        b.setFont(Font.font("Segoe UI", 12));
        b.setTextFill(TEXT);
        b.setStyle("-fx-background-color: #2D2D37; -fx-background-radius: 6; -fx-cursor: hand; -fx-padding: 8 18;");
        b.setOnMouseEntered(e -> b.setStyle("-fx-background-color: rgba(99,179,237,0.3); -fx-background-radius: 6; -fx-cursor: hand; -fx-padding: 8 18;"));
        b.setOnMouseExited(e -> b.setStyle("-fx-background-color: #2D2D37; -fx-background-radius: 6; -fx-cursor: hand; -fx-padding: 8 18;"));
        return b;
    }

    private VBox createContent() {
        VBox v = new VBox();
        v.setPadding(new Insets(16, 20, 16, 20));
        v.setStyle("-fx-background-color: #0E0E14; -fx-background-radius: 12;");

        tabBar = new HBox(0);
        tabBar.setPadding(new Insets(0, 0, 12, 0));

        String[] tabs = {"Processes", "Configs", "Logs"};
        for (int i = 0; i < tabs.length; i++) {
            final int idx = i;
            Label lbl = new Label(tabs[i]);
            lbl.setFont(Font.font("Segoe UI", 12));
            lbl.setPadding(new Insets(8, 16, 8, 16));
            lbl.setStyle(i == 0 ? "-fx-text-fill: #F0F0F5;" : "-fx-text-fill: #787887; -fx-cursor: hand;");
            lbl.setOnMouseClicked(e -> switchTab(idx));
            tabBar.getChildren().add(lbl);
        }

        contentPane = new StackPane();
        contentPane.getChildren().add(buildProcessTab());
        VBox.setVgrow(contentPane, Priority.ALWAYS);

        v.getChildren().addAll(tabBar, contentPane);
        return v;
    }

    private void switchTab(int idx) {
        for (int i = 0; i < tabBar.getChildren().size(); i++) {
            Label l = (Label) tabBar.getChildren().get(i);
            l.setStyle(i == idx ? "-fx-text-fill: #F0F0F5;" : "-fx-text-fill: #787887; -fx-cursor: hand;");
        }
        contentPane.getChildren().clear();
        switch (idx) {
            case 0: contentPane.getChildren().add(buildProcessTab()); break;
            case 1: contentPane.getChildren().add(buildConfigTab()); break;
            case 2: contentPane.getChildren().add(buildLogTab()); break;
        }
    }

    private VBox buildProcessTab() {
        VBox p = new VBox(12);

        processList = new ListView<>();
        processList.setPrefHeight(320);
        processList.getItems().addAll(loadConfigs());

        String cellStyle = "-fx-background-color: transparent; -fx-text-fill: #F0F0F5; -fx-font-size: 13px; -fx-padding: 8 12;";
        processList.setCellFactory(lv -> new ListCell<String>() {
            @Override protected void updateItem(String item, boolean empty) {
                super.updateItem(item, empty);
                setText(empty || item == null ? null : item);
                setStyle(cellStyle);
            }
        });
        processList.setStyle("-fx-background-color: #14141C; -fx-border-color: #2A2A38; -fx-border-radius: 8; -fx-background-radius: 8;");
        VBox.setVgrow(processList, Priority.ALWAYS);

        HBox btns = new HBox(10);
        Button add = accentBtn("Add Game");
        add.setOnAction(e -> showAddDialog());

        Button rem = accentBtn("Remove");
        rem.setOnAction(e -> {
            String sel = processList.getSelectionModel().getSelectedItem();
            if (sel != null) {
                new File(CONFIG_DIR, sel + ".json").delete();
                processList.getItems().remove(sel);
            }
        });

        Button ref = accentBtn("Refresh");
        ref.setOnAction(e -> { processList.getItems().clear(); processList.getItems().addAll(loadConfigs()); });

        btns.getChildren().addAll(add, rem, ref);
        p.getChildren().addAll(processList, btns);
        return p;
    }

    private VBox buildConfigTab() {
        VBox p = new VBox(10);
        if (processList == null || processList.getSelectionModel().getSelectedItem() == null) {
            Label l = new Label("Select a process first");
            l.setTextFill(DIM);
            p.getChildren().add(l);
            return p;
        }
        String sel = processList.getSelectionModel().getSelectedItem();
        File f = new File(CONFIG_DIR, sel + ".json");
        if (!f.exists()) { p.getChildren().add(new Label("Not found")); return p; }
        try {
            String content = new String(Files.readAllBytes(f.toPath()));
            TextArea ed = new TextArea(content);
            ed.setFont(Font.font("Consolas", 13));
            ed.setStyle("-fx-control-inner-background: #14141C; -fx-text-fill: #F0F0F5; -fx-border-color: #2A2A38; -fx-border-radius: 8; -fx-background-radius: 8;");
            ed.setPrefHeight(320);
            VBox.setVgrow(ed, Priority.ALWAYS);

            Button save = accentBtn("Save");
            save.setOnAction(e -> {
                try { Files.write(f.toPath(), ed.getText().getBytes()); } catch (Exception ex) { }
            });

            Label name = new Label(sel + ".json");
            name.setFont(Font.font("Segoe UI", FontWeight.BOLD, 13));
            p.getChildren().addAll(name, ed, save);
        } catch (Exception e) {
            p.getChildren().add(new Label("Error"));
        }
        return p;
    }

    private VBox buildLogTab() {
        VBox p = new VBox(10);
        logArea = new TextArea();
        logArea.setEditable(false);
        logArea.setFont(Font.font("Consolas", 12));
        logArea.setStyle("-fx-control-inner-background: #14141C; -fx-text-fill: #F0F0F5; -fx-border-color: #2A2A38; -fx-border-radius: 8; -fx-background-radius: 8;");
        logArea.setPrefHeight(320);
        VBox.setVgrow(logArea, Priority.ALWAYS);

        Button clr = accentBtn("Clear");
        clr.setOnAction(e -> logArea.clear());
        p.getChildren().addAll(logArea, clr);
        return p;
    }

    private HBox createStatusBar() {
        HBox bar = new HBox();
        bar.setAlignment(Pos.CENTER_LEFT);
        bar.setPadding(new Insets(10, 24, 10, 24));
        bar.setStyle("-fx-background-color: #0E0E14;");

        statusLabel = new Label("Idle");
        statusLabel.setFont(Font.font("Segoe UI", 11));
        statusLabel.setTextFill(DIM);

        Region sp = new Region();
        HBox.setHgrow(sp, Priority.ALWAYS);

        Label v = new Label("v" + APP_VERSION);
        v.setFont(Font.font("Segoe UI", 10));
        v.setTextFill(DIM);

        bar.getChildren().addAll(statusLabel, sp, v);
        return bar;
    }

    private void showAddDialog() {
        Stage d = new Stage();
        d.initStyle(StageStyle.UNDECORATED);

        final double[] off = new double[2];

        VBox root = new VBox(15);
        root.setPadding(new Insets(25));
        root.setStyle("-fx-background-color: #16161E; -fx-background-radius: 12;");
        root.setOnMousePressed(e -> { off[0] = e.getScreenX() - d.getX(); off[1] = e.getScreenY() - d.getY(); });
        root.setOnMouseDragged(e -> { d.setX(e.getScreenX() - off[0]); d.setY(e.getScreenY() - off[1]); });

        Text title = new Text("Add Game");
        title.setFont(Font.font("Segoe UI", FontWeight.BOLD, 16));
        title.setFill(TEXT);

        TextField pf = makeField("SFHR.exe");
        TextField af = makeField("1404735389603860503");

        Text err = new Text("");
        err.setFill(RED);

        Button cancel = accentBtn("Cancel");
        cancel.setOnAction(e -> d.close());

        Button add = coloredBtn("Add", ACCENT);
        add.setOnAction(e -> {
            String p = pf.getText().trim();
            String a = af.getText().trim();
            if (p.isEmpty()) { err.setText("Enter process name"); return; }
            if (!p.toLowerCase().endsWith(".exe")) { err.setText("Must end with .exe"); return; }
            if (a.isEmpty()) { err.setText("Enter App ID"); return; }
            if (!a.matches("\\d{17,20}")) { err.setText("ID: 17-20 digits"); return; }
            saveConfig(p, a);
            processList.getItems().clear();
            processList.getItems().addAll(loadConfigs());
            d.close();
        });

        HBox btns = new HBox(10);
        btns.setAlignment(Pos.CENTER_RIGHT);
        btns.getChildren().addAll(cancel, add);

        root.getChildren().addAll(title, pf, af, err, btns);

        Scene s = new Scene(root, 400, 260);
        d.setScene(s);
        d.centerOnScreen();
        d.show();
    }

    private TextField makeField(String prompt) {
        TextField f = new TextField();
        f.setPromptText(prompt);
        f.setFont(Font.font("Segoe UI", 13));
        f.setStyle("-fx-background-color: #1E1E28; -fx-text-fill: #F0F0F5; -fx-prompt-text-fill: #787887; -fx-border-color: #2A2A38; -fx-border-radius: 6; -fx-background-radius: 6; -fx-padding: 10;");
        return f;
    }

    private void startEngine() {
        try {
            String exe = findExe();
            if (exe == null) { log("Engine not found"); return; }
            new File(CONFIG_DIR).mkdirs();
            ProcessBuilder pb = new ProcessBuilder(exe, "engine");
            pb.directory(new File(CONFIG_DIR));
            pb.redirectErrorStream(true);
            engineProcess = pb.start();

            new Thread(() -> {
                try (BufferedReader r = new BufferedReader(new InputStreamReader(engineProcess.getInputStream()))) {
                    String line;
                    while ((line = r.readLine()) != null) {
                        final String l = line;
                        Platform.runLater(() -> log(l));
                    }
                } catch (IOException e) { }
            }, "log-reader").start();

            statusLabel.setText("Running");
            statusLabel.setTextFill(GREEN);
            log("Engine started");
        } catch (Exception e) {
            log("Error: " + e.getMessage());
        }
    }

    private void stopEngine() {
        if (engineProcess != null && engineProcess.isAlive()) {
            engineProcess.destroy();
            statusLabel.setText("Stopped");
            statusLabel.setTextFill(RED);
            log("Engine stopped");
        }
    }

    private String findExe() {
        for (String p : new String[]{"xgs.exe", "core\\target\\release\\xgs.exe", "..\\xgs.exe"})
            if (new File(p).exists()) return new File(p).getAbsolutePath();
        return null;
    }

    private void log(String msg) {
        if (logArea != null) logArea.appendText(msg + "\n");
    }

    private String[] loadConfigs() {
        File dir = new File(CONFIG_DIR);
        if (!dir.exists()) return new String[0];
        File[] files = dir.listFiles((d, n) -> n.endsWith(".json"));
        if (files == null) return new String[0];
        String[] r = new String[files.length];
        for (int i = 0; i < files.length; i++) r[i] = files[i].getName().replace(".json", "");
        return r;
    }

    private void saveConfig(String process, String appId) {
        try {
            new File(CONFIG_DIR).mkdirs();
            String json = "{\"process_name\":\"" + process + "\",\"discord_app_id\":\"" + appId
                + "\",\"scan_type\":\"offsets\",\"requires_elevation\":false,\"pointers\":{},\"rpc_template\":{\"details\":\"Playing\",\"state\":\"In Game\"}}";
            Files.write(new File(CONFIG_DIR, process.replace(".exe", "").toLowerCase() + ".json").toPath(), json.getBytes());
            log("Added: " + process);
        } catch (Exception e) { log("Error: " + e.getMessage()); }
    }

    private void centerOnScreen(Stage s) {
        var b = Screen.getPrimary().getBounds();
        s.setX((b.getWidth() - s.getWidth()) / 2);
        s.setY((b.getHeight() - s.getHeight()) / 2);
    }

    public static void main(String[] args) { launch(args); }
}
