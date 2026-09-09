package com.gamepresence.gui;

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
import java.io.BufferedReader;
import java.io.File;
import java.io.IOException;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.HashMap;
import java.util.Map;

import javafx.animation.FadeTransition;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.control.ChoiceBox;
import javafx.scene.control.Label;
import javafx.scene.control.ListCell;
import javafx.scene.control.ListView;
import javafx.scene.control.ProgressIndicator;
import javafx.scene.control.TextArea;
import javafx.scene.control.TextField;
import javafx.scene.image.Image;
import javafx.scene.layout.Background;
import javafx.scene.layout.BorderPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.StackPane;
import javafx.scene.layout.VBox;
import javafx.scene.paint.Color;
import javafx.scene.text.Font;
import javafx.scene.text.FontWeight;
import javafx.scene.text.Text;
import javafx.stage.Modality;
import javafx.stage.Screen;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import javafx.util.Duration;

public class Main extends Application {

    private static final String APP_NAME    = "XGameStats";
    private static final String APP_VERSION = "0.6.0";
    private static final String CONFIG_DIR  = System.getProperty("user.home")
            + File.separator + "AppData" + File.separator + "Local"
            + File.separator + "XGameStats";

    private static final Color ACCENT   = Color.rgb(88, 166, 255);
    private static final Color TEXT     = Color.rgb(220, 220, 230);
    private static final Color DIM      = Color.rgb(130, 130, 145);
    private static final Color GREEN    = Color.rgb(66, 185, 130);
    private static final Color RED      = Color.rgb(235, 80, 80);

    private Stage primaryStage;
    private Process          engineProcess;
    private ListView<String> processList;
    private TextArea         logArea;
    private boolean          userScrolledUp = false;
    private Label            statusLabel;
    private StackPane        contentPane;
    private HBox             tabBar;
    private Text             subtitleText;

    private TrayIcon trayIcon;
    private double dragOffsetX, dragOffsetY;

    private String language = "en";
    private final Map<String, Map<String, String>> translations = new HashMap<>();
    private String globalAppId = "";

    public Main() {}

    @Override
    public void start(Stage stage) {
        primaryStage = stage;
        loadSettings();
        loadTranslations();
        loadIcon(stage);
        Platform.setImplicitExit(false);
        installTray(stage);
        showSplash(stage);
    }

    private void loadSettings() {
        File f = new File(CONFIG_DIR, "settings.json");
        if (!f.exists()) return;
        try {
            String raw = new String(Files.readAllBytes(f.toPath()), StandardCharsets.UTF_8);
            String lid = extractJsonString(raw, "language");
            String aid = extractJsonString(raw, "discord_app_id");
            if (lid != null && !lid.isEmpty()) language = lid;
            if (aid != null) globalAppId = aid;
        } catch (IOException e) { }
    }

    private void saveSettings() {
        try {
            new File(CONFIG_DIR).mkdirs();
            String json = "{\"discord_app_id\":\"" + esc(globalAppId)
                + "\",\"language\":\"" + esc(language) + "\"}";
            Files.write(new File(CONFIG_DIR, "settings.json").toPath(), json.getBytes(StandardCharsets.UTF_8));
        } catch (IOException e) { }
    }

    private void loadTranslations() {
        String[] paths = {"translations.json", "../translations.json", "../../translations.json"};
        for (String p : paths) {
            File f = new File(p);
            if (f.exists()) {
                try {
                    String raw = new String(Files.readAllBytes(f.toPath()), StandardCharsets.UTF_8);
                    parseTranslations(raw);
                    return;
                } catch (IOException e) { }
            }
        }
    }

    private void parseTranslations(String json) {
        String[] langs = {"en", "ru"};
        for (String lang : langs) {
            int start = json.indexOf("\"" + lang + "\"");
            if (start < 0) continue;
            int objStart = json.indexOf('{', start);
            if (objStart < 0) continue;
            int depth = 0; int objEnd = objStart;
            for (int i = objStart; i < json.length(); i++) {
                char c = json.charAt(i);
                if (c == '{') depth++;
                else if (c == '}') { depth--; if (depth == 0) { objEnd = i; break; } }
            }
            String block = json.substring(objStart, objEnd + 1);
            Map<String, String> map = new HashMap<>();
            int p = 0;
            while (p < block.length()) {
                int kq = block.indexOf('"', p);
                if (kq < 0) break;
                int kq2 = block.indexOf('"', kq + 1);
                if (kq2 < 0) break;
                String key = block.substring(kq + 1, kq2);
                int colon = block.indexOf(':', kq2);
                if (colon < 0) break;
                int vq = block.indexOf('"', colon + 1);
                if (vq < 0) break;
                int vq2 = vq + 1;
                while (vq2 < block.length()) {
                    char c = block.charAt(vq2);
                    if (c == '\\') { vq2 += 2; continue; }
                    if (c == '"') break;
                    vq2++;
                }
                String val = block.substring(vq + 1, vq2);
                val = val.replace("\\n", "\n").replace("\\\"", "\"");
                map.put(key, val);
                p = vq2 + 1;
            }
            translations.put(lang, map);
        }
    }

    private String t(String key) {
        Map<String, String> m = translations.get(language);
        if (m != null && m.containsKey(key)) return m.get(key);
        Map<String, String> en = translations.get("en");
        if (en != null && en.containsKey(key)) return en.get(key);
        return key;
    }

    private String esc(String s) {
        if (s == null) return "";
        return s.replace("\\", "\\\\").replace("\"", "\\\"");
    }

    private String extractJsonString(String json, String key) {
        String search = "\"" + key + "\"";
        int idx = json.indexOf(search);
        if (idx < 0) return null;
        int q1 = json.indexOf('"', idx + search.length());
        if (q1 < 0) return null;
        int q2 = json.indexOf('"', q1 + 1);
        if (q2 < 0) return null;
        return json.substring(q1 + 1, q2);
    }

    private void loadIcon(Stage stage) {
        String[] paths = {"assets/logo.png", "../assets/logo.png", "../../assets/logo.png"};
        for (String p : paths) {
            File f = new File(p);
            if (f.exists()) {
                stage.getIcons().add(new Image(f.toURI().toString()));
                return;
            }
        }
    }

    private void showSplash(Stage stage) {
        StackPane root = new StackPane();
        root.setStyle("-fx-background-color: #0A0A0E;");

        VBox box = new VBox(20);
        box.setAlignment(Pos.CENTER);

        Text t = new Text(APP_NAME);
        t.setFont(Font.font("Segoe UI", FontWeight.BOLD, 48));
        t.setFill(TEXT);

        Text v = new Text("v" + APP_VERSION);
        v.setFont(Font.font("Segoe UI", 13));
        v.setFill(DIM);

        ProgressIndicator sp = new ProgressIndicator();
        sp.setStyle("-fx-progress-color: #58A6FF;");
        sp.setMaxSize(40, 40);

        box.getChildren().addAll(t, v, sp);
        root.getChildren().add(box);

        Scene s = new Scene(root, 480, 320);
        stage.initStyle(StageStyle.UNDECORATED);
        stage.setScene(s);
        stage.show();
        centerOnScreen(stage);

        FadeTransition fadeIn = new FadeTransition(Duration.millis(400), root);
        fadeIn.setFromValue(0.0);
        fadeIn.setToValue(1.0);
        fadeIn.play();

        new Thread(() -> {
            try { Thread.sleep(1800); } catch (InterruptedException e) { Thread.currentThread().interrupt(); }
            Platform.runLater(() -> { stage.close(); showMain(stage); });
        }).start();
    }

    private void showMain(Stage stage) {
        BorderPane root = new BorderPane();
        root.setStyle("-fx-background-color: #0A0A0E;");

        HBox header = createHeader(stage);
        root.setTop(header);
        root.setCenter(createContent());
        root.setBottom(createStatusBar());

        Scene s = new Scene(root, 820, 560);
        stage.setScene(s);
        stage.show();
        centerOnScreen(stage);

        FadeTransition fadeIn = new FadeTransition(Duration.millis(300), root);
        fadeIn.setFromValue(0.0);
        fadeIn.setToValue(1.0);
        fadeIn.play();

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
        h.setStyle("-fx-background-color: #0C0C10;");

        VBox tb = new VBox(2);
        Text t = new Text(APP_NAME);
        t.setFont(Font.font("Segoe UI", FontWeight.BOLD, 20));
        t.setFill(ACCENT);
        subtitleText = new Text(t("subtitle"));
        subtitleText.setFont(Font.font("Segoe UI", 11));
        subtitleText.setFill(DIM);
        tb.getChildren().addAll(t, subtitleText);

        Region spacer = new Region();
        HBox.setHgrow(spacer, Priority.ALWAYS);

        Button btnMin = windowBtn("\u2212");
        btnMin.setOnAction(e -> minimizeToTray(stage));

        Button btnClose = windowBtn("\u00d7");
        btnClose.setOnMouseEntered(ev -> btnClose.setTextFill(RED));
        btnClose.setOnMouseExited(ev  -> btnClose.setTextFill(DIM));
        btnClose.setOnAction(e -> minimizeToTray(stage));

        HBox btns = new HBox(8);
        btns.setAlignment(Pos.CENTER_RIGHT);
        btns.getChildren().addAll(btnMin, btnClose);
        btns.setOnMousePressed(javafx.event.Event::consume);
        btns.setOnMouseDragged(javafx.event.Event::consume);

        h.getChildren().addAll(tb, spacer, btns);
        return h;
    }

    private void installTray(Stage stage) {
        if (!SystemTray.isSupported()) return;
        SystemTray tray = SystemTray.getSystemTray();

        PopupMenu menu = new PopupMenu();

        MenuItem itemShow  = new MenuItem(t("tray_show"));
        itemShow.addActionListener(e -> Platform.runLater(() -> showFromTray(stage)));

        MenuItem itemStart = new MenuItem(t("tray_start"));
        itemStart.addActionListener(e -> Platform.runLater(this::startEngine));

        MenuItem itemStop  = new MenuItem(t("tray_stop"));
        itemStop.addActionListener(e -> Platform.runLater(this::stopEngine));

        MenuItem itemExit  = new MenuItem(t("tray_exit"));
        itemExit.addActionListener(e -> Platform.runLater(this::exitApp));

        menu.add(itemShow);
        menu.addSeparator();
        menu.add(itemStart);
        menu.add(itemStop);
        menu.addSeparator();
        menu.add(itemExit);

        java.awt.Image trayImage = loadTrayImage();
        trayIcon = new TrayIcon(trayImage, APP_NAME, menu);
        trayIcon.setImageAutoSize(true);
        trayIcon.addMouseListener(new MouseAdapter() {
            @Override public void mouseClicked(MouseEvent e) {
                if (e.getClickCount() >= 2) Platform.runLater(() -> showFromTray(stage));
            }
        });

        try { tray.add(trayIcon); } catch (AWTException ex) { }
    }

    private java.awt.Image loadTrayImage() {
        String[] paths = {"assets/logo.png", "../assets/logo.png", "../../assets/logo.png"};
        for (String p : paths) {
            File f = new File(p);
            if (f.exists()) {
                try {
                    java.awt.image.BufferedImage img = javax.imageio.ImageIO.read(f);
                    if (img != null) {
                        java.awt.image.BufferedImage resized = new java.awt.image.BufferedImage(256, 256, java.awt.image.BufferedImage.TYPE_INT_ARGB);
                        java.awt.Graphics2D g = resized.createGraphics();
                        g.setRenderingHint(java.awt.RenderingHints.KEY_INTERPOLATION, java.awt.RenderingHints.VALUE_INTERPOLATION_BILINEAR);
                        g.setRenderingHint(java.awt.RenderingHints.KEY_RENDERING, java.awt.RenderingHints.VALUE_RENDER_QUALITY);
                        g.setRenderingHint(java.awt.RenderingHints.KEY_ANTIALIASING, java.awt.RenderingHints.VALUE_ANTIALIAS_ON);
                        g.drawImage(img, 0, 0, 256, 256, null);
                        g.dispose();
                        return resized;
                    }
                } catch (IOException | IllegalArgumentException e) { }
            }
        }
        return buildTrayImage();
    }

    private void minimizeToTray(Stage stage) {
        stage.hide();
        showNotification(t("tray_minimized"));
    }

    private void showNotification(String message) {
        Stage popup = new Stage();
        popup.initStyle(StageStyle.UNDECORATED);
        popup.setAlwaysOnTop(true);
        popup.initStyle(StageStyle.TRANSPARENT);

        VBox root = new VBox(8);
        root.setPadding(new Insets(14, 18, 14, 18));
        root.setStyle("-fx-background-color: #141418; -fx-border-color: #1C1C24;");

        Text title = new Text(APP_NAME);
        title.setFont(Font.font("Segoe UI", FontWeight.BOLD, 13));
        title.setFill(ACCENT);

        Text msg = new Text(message);
        msg.setFont(Font.font("Segoe UI", 12));
        msg.setFill(TEXT);
        msg.setWrappingWidth(280);

        root.getChildren().addAll(title, msg);

        Scene sc = new Scene(root, 320, 100);
        sc.setFill(Color.TRANSPARENT);
        popup.setScene(sc);

        var bounds = Screen.getPrimary().getBounds();
        popup.setX(bounds.getMaxX() - 340);
        popup.setY(bounds.getMaxY() - 140);

        popup.show();

        FadeTransition fadeIn = new FadeTransition(Duration.millis(300), root);
        fadeIn.setFromValue(0.0);
        fadeIn.setToValue(1.0);
        fadeIn.play();

        new Thread(() -> {
            try { Thread.sleep(3000); } catch (InterruptedException e) { Thread.currentThread().interrupt(); }
            Platform.runLater(() -> {
                FadeTransition fadeOut = new FadeTransition(Duration.millis(400), root);
                fadeOut.setFromValue(1.0);
                fadeOut.setToValue(0.0);
                fadeOut.setOnFinished(e2 -> popup.close());
                fadeOut.play();
            });
        }).start();
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
        int sz = 256;
        BufferedImage img = new BufferedImage(sz, sz, BufferedImage.TYPE_INT_ARGB);
        Graphics2D g = img.createGraphics();
        g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
        g.setRenderingHint(RenderingHints.KEY_RENDERING, RenderingHints.VALUE_RENDER_QUALITY);
        g.setColor(new java.awt.Color(10, 10, 14));
        g.fillOval(8, 8, sz - 16, sz - 16);
        g.setColor(new java.awt.Color(88, 166, 255));
        g.setStroke(new BasicStroke(8f));
        g.drawOval(16, 16, sz - 32, sz - 32);
        g.setColor(new java.awt.Color(220, 220, 230));
        g.setFont(new java.awt.Font("Segoe UI", java.awt.Font.BOLD, 140));
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
        b.setOnMouseEntered(e -> b.setStyle(colorStyle(color, 1.15)));
        b.setOnMouseExited(e  -> b.setStyle(colorStyle(color, 1.0)));
        return b;
    }

    private String colorStyle(Color c, double f) {
        int r  = Math.min(255, (int)(c.getRed()   * 255 * f));
        int g  = Math.min(255, (int)(c.getGreen() * 255 * f));
        int bl = Math.min(255, (int)(c.getBlue()  * 255 * f));
        return "-fx-background-color: rgb(" + r + "," + g + "," + bl + "); -fx-cursor: hand; -fx-padding: 8 20;";
    }

    private Button accentBtn(String text) {
        Button b = new Button(text);
        b.setFont(Font.font("Segoe UI", 12));
        b.setTextFill(TEXT);
        b.setStyle("-fx-background-color: #181820; -fx-cursor: hand; -fx-padding: 8 18;");
        b.setOnMouseEntered(e -> b.setStyle("-fx-background-color: rgba(88,166,255,0.2); -fx-cursor: hand; -fx-padding: 8 18;"));
        b.setOnMouseExited(e -> b.setStyle("-fx-background-color: #181820; -fx-cursor: hand; -fx-padding: 8 18;"));
        return b;
    }

    private VBox createContent() {
        VBox v = new VBox();
        v.setPadding(new Insets(16, 20, 16, 20));
        v.setStyle("-fx-background-color: #0D0D12;");

        tabBar = new HBox(0);
        tabBar.setPadding(new Insets(0, 0, 12, 0));

        String[] tabs = {t("tab_processes"), t("tab_configs"), t("tab_logs"), t("tab_settings"), t("tab_faq")};
        for (int i = 0; i < tabs.length; i++) {
            final int idx = i;
            Label lbl = new Label(tabs[i]);
            lbl.setFont(Font.font("Segoe UI", 12));
            lbl.setPadding(new Insets(8, 16, 8, 16));
            lbl.setStyle(i == 0 ? "-fx-text-fill: #DCDCE6;" : "-fx-text-fill: #828291; -fx-cursor: hand;");
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
            l.setStyle(i == idx ? "-fx-text-fill: #DCDCE6;" : "-fx-text-fill: #828291; -fx-cursor: hand;");
        }

        javafx.scene.Node newContent;
        switch (idx) {
            case 0 -> newContent = buildProcessTab();
            case 1 -> newContent = buildConfigTab();
            case 2 -> newContent = buildLogTab();
            case 3 -> newContent = buildSettingsTab();
            case 4 -> newContent = buildFaqTab();
            default -> newContent = buildProcessTab();
        }

        contentPane.getChildren().clear();
        contentPane.getChildren().add(newContent);

        FadeTransition ft = new FadeTransition(Duration.millis(150), newContent);
        ft.setFromValue(0.0);
        ft.setToValue(1.0);
        ft.play();
    }

    private VBox buildProcessTab() {
        VBox p = new VBox(12);

        processList = new ListView<>();
        processList.setPrefHeight(320);
        processList.getSelectionModel().setSelectionMode(javafx.scene.control.SelectionMode.SINGLE);
        processList.getItems().addAll(loadConfigs());

        processList.setCellFactory(lv -> new ListCell<String>() {
            @Override protected void updateItem(String item, boolean empty) {
                super.updateItem(item, empty);
                setText(empty || item == null ? null : item);
                if (empty || item == null) {
                    setStyle("-fx-background-color: transparent; -fx-text-fill: #DCDCE6; -fx-font-size: 13px; -fx-padding: 8 12;");
                } else if (isSelected()) {
                    setStyle("-fx-background-color: #161620; -fx-text-fill: #DCDCE6; -fx-font-size: 13px; -fx-padding: 8 12;");
                } else {
                    setStyle("-fx-background-color: transparent; -fx-text-fill: #DCDCE6; -fx-font-size: 13px; -fx-padding: 8 12;");
                }
            }
        });
        processList.setStyle("-fx-background-color: #0E0E12; -fx-border-color: #1C1C24;");
        processList.getFocusModel().focusedIndexProperty().addListener((obs, old, val) -> processList.refresh());
        processList.getSelectionModel().selectedItemProperty().addListener((obs, old, val) -> processList.refresh());
        VBox.setVgrow(processList, Priority.ALWAYS);

        HBox btns = new HBox(8);

        Button start = coloredBtn(t("btn_start"), GREEN);
        start.setOnAction(e -> {
            if (processList.getSelectionModel().getSelectedItem() == null) {
                showConfirmDialog(primaryStage, t("select_process_first"), () -> {});
                return;
            }
            startEngine();
        });

        Button stop = coloredBtn(t("btn_stop"), RED);
        stop.setOnAction(e -> stopEngine());

        Region spacer = new Region();
        HBox.setHgrow(spacer, Priority.ALWAYS);

        Button add = accentBtn(t("btn_add"));
        add.setOnAction(e -> showAddDialog());

        Button rem = accentBtn(t("btn_remove"));
        rem.setOnAction(e -> {
            String sel = processList.getSelectionModel().getSelectedItem();
            if (sel != null) {
                new File(CONFIG_DIR, sel + ".json").delete();
                processList.getItems().remove(sel);
                log(t("log_removed").replace("{0}", sel));
            }
        });

        Button ref = accentBtn(t("btn_refresh"));
        ref.setOnAction(e -> { processList.getItems().clear(); processList.getItems().addAll(loadConfigs()); });

        btns.getChildren().addAll(start, stop, spacer, add, rem, ref);
        p.getChildren().addAll(processList, btns);
        return p;
    }

    private VBox buildConfigTab() {
        VBox p = new VBox(10);
        if (processList == null || processList.getSelectionModel().getSelectedItem() == null) {
            Label l = new Label(t("select_process_first"));
            l.setTextFill(DIM);
            p.getChildren().add(l);
            return p;
        }
        String sel = processList.getSelectionModel().getSelectedItem();
        File f = new File(CONFIG_DIR, sel + ".json");
        if (!f.exists()) { p.getChildren().add(new Label(t("config_not_found"))); return p; }
        try {
            String content = new String(Files.readAllBytes(f.toPath()), StandardCharsets.UTF_8);
            TextArea ed = new TextArea(content);
            ed.setFont(Font.font("Consolas", 13));
            ed.setStyle("-fx-control-inner-background: #0E0E12; -fx-text-fill: #DCDCE6; -fx-border-color: #1C1C24;");
            ed.setPrefHeight(320);
            VBox.setVgrow(ed, Priority.ALWAYS);

            Button save = accentBtn(t("btn_save"));
            save.setOnAction(e -> writeFile(f.toPath(), ed.getText().getBytes(StandardCharsets.UTF_8)));

            Label name = new Label(sel + ".json");
            name.setFont(Font.font("Segoe UI", FontWeight.BOLD, 13));
            p.getChildren().addAll(name, ed, save);
        } catch (IOException e) {
            p.getChildren().add(new Label(t("error")));
        }
        return p;
    }

    private VBox buildLogTab() {
        VBox p = new VBox(10);
        logArea = new TextArea();
        logArea.setEditable(false);
        logArea.setFont(Font.font("Consolas", 12));
        logArea.setStyle("-fx-control-inner-background: #0E0E12; -fx-text-fill: #DCDCE6; -fx-border-color: #1C1C24;");
        logArea.setPrefHeight(320);
        logArea.setWrapText(false);

        logArea.textProperty().addListener((obs, old, val) -> {
            if (!userScrolledUp) {
                logArea.setScrollTop(Double.MAX_VALUE);
            }
        });

        logArea.scrollTopProperty().addListener((obs, old, val) -> {
            double max = logArea.heightProperty().doubleValue();
            double top = val.doubleValue();
            userScrolledUp = top < max && top > 0;
        });
        VBox.setVgrow(logArea, Priority.ALWAYS);

        Button clr = accentBtn(t("btn_clear"));
        clr.setOnAction(e -> logArea.clear());
        p.getChildren().addAll(logArea, clr);
        return p;
    }

    private VBox buildSettingsTab() {
        VBox p = new VBox(16);
        p.setPadding(new Insets(4, 0, 0, 0));

        Text title = new Text(t("settings_title"));
        title.setFont(Font.font("Segoe UI", FontWeight.BOLD, 16));
        title.setFill(TEXT);

        Text discordLbl = new Text(t("settings_discord_id"));
        discordLbl.setFont(Font.font("Segoe UI", 13));
        discordLbl.setFill(TEXT);

        Text discordDesc = new Text(t("settings_discord_desc"));
        discordDesc.setFont(Font.font("Segoe UI", 11));
        discordDesc.setFill(DIM);

        TextField discordField = makeField("1234567890");
        discordField.setText(globalAppId);

        Text langLbl = new Text(t("settings_language"));
        langLbl.setFont(Font.font("Segoe UI", 13));
        langLbl.setFill(TEXT);

        ChoiceBox<String> langBox = new ChoiceBox<>();
        langBox.getItems().addAll("English", "\u0420\u0443\u0441\u0441\u043a\u0438\u0439");
        langBox.setValue("en".equals(language) ? "English" : "\u0420\u0443\u0441\u0441\u043a\u0438\u0439");
        langBox.setStyle("-fx-background-color: #0A0A0E; -fx-border-color: #1C1C28; -fx-padding: 4;");

        Button apply = accentBtn(t("btn_apply"));
        apply.setOnAction(e -> {
            String newAppId = discordField.getText().trim();
            String newLang = "English".equals(langBox.getValue()) ? "en" : "ru";

            if (newAppId.isEmpty()) {
                showConfirmDialog(primaryStage, t("err_enter_id"), () -> {});
                return;
            }

            if (!newAppId.matches("\\d{17,20}")) {
                showConfirmDialog(primaryStage, t("err_invalid_id"), () -> {});
                return;
            }

            globalAppId = newAppId;
            language = newLang;
            saveSettings();
            rebuildUI();
            showConfirmDialog(primaryStage, t("settings_saved"), () -> {});
        });

        Region spacer = new Region();
        VBox.setVgrow(spacer, Priority.ALWAYS);

        p.getChildren().addAll(title, discordLbl, discordDesc, discordField, langLbl, langBox, spacer, apply);
        return p;
    }

    private VBox buildFaqTab() {
        VBox p = new VBox(16);
        p.setPadding(new Insets(4, 0, 0, 0));

        Text title = new Text(t("faq_title"));
        title.setFont(Font.font("Segoe UI", FontWeight.BOLD, 16));
        title.setFill(TEXT);

        String[][] faqs = {
            {t("faq_q1"), t("faq_a1")},
            {t("faq_q2"), t("faq_a2")},
            {t("faq_q3"), t("faq_a3")},
            {t("faq_q4"), t("faq_a4")}
        };

        VBox items = new VBox(12);
        for (String[] faq : faqs) {
            VBox item = new VBox(4);
            item.setPadding(new Insets(12));
            item.setStyle("-fx-background-color: #0E0E12; -fx-border-color: #1C1C24;");

            Text q = new Text(faq[0]);
            q.setFont(Font.font("Segoe UI", FontWeight.BOLD, 13));
            q.setFill(ACCENT);

            Text a = new Text(faq[1]);
            a.setFont(Font.font("Segoe UI", 12));
            a.setFill(TEXT);
            a.setWrappingWidth(700);

            item.getChildren().addAll(q, a);
            items.getChildren().add(item);
        }

        Region spacer = new Region();
        VBox.setVgrow(spacer, Priority.ALWAYS);

        p.getChildren().addAll(title, items, spacer);
        return p;
    }

    private void rebuildUI() {
        if (primaryStage == null) return;
        Scene s = primaryStage.getScene();
        if (s == null) return;
        BorderPane root = (BorderPane) s.getRoot();
        root.getChildren().clear();

        HBox header = createHeader(primaryStage);
        root.setTop(header);
        root.setCenter(createContent());
        root.setBottom(createStatusBar());

        header.setOnMousePressed(e -> {
            dragOffsetX = e.getScreenX() - primaryStage.getX();
            dragOffsetY = e.getScreenY() - primaryStage.getY();
        });
        header.setOnMouseDragged(e -> {
            primaryStage.setX(e.getScreenX() - dragOffsetX);
            primaryStage.setY(e.getScreenY() - dragOffsetY);
        });

        primaryStage.setOnCloseRequest(e -> {
            e.consume();
            minimizeToTray(primaryStage);
        });
    }

    private void showConfirmDialog(Stage owner, String message, Runnable onOk) {
        Stage d = new Stage();
        d.initStyle(StageStyle.UNDECORATED);
        d.initModality(Modality.APPLICATION_MODAL);
        d.initOwner(owner);

        final double[] off = new double[2];

        VBox root = new VBox(15);
        root.setPadding(new Insets(25));
        root.setStyle("-fx-background-color: #0E0E12; -fx-border-color: #1C1C24;");
        root.setOnMousePressed(e -> { off[0] = e.getScreenX() - d.getX(); off[1] = e.getScreenY() - d.getY(); });
        root.setOnMouseDragged(e -> { d.setX(e.getScreenX() - off[0]); d.setY(e.getScreenY() - off[1]); });

        Text msg = new Text(message);
        msg.setFont(Font.font("Segoe UI", 13));
        msg.setFill(TEXT);
        msg.setWrappingWidth(320);

        Button ok = coloredBtn("OK", ACCENT);
        ok.setOnAction(e -> { d.close(); onOk.run(); });

        HBox btns = new HBox(10);
        btns.setAlignment(Pos.CENTER_RIGHT);
        btns.getChildren().add(ok);

        root.getChildren().addAll(msg, btns);

        Scene sc = new Scene(root, 380, 160);
        sc.setFill(Color.TRANSPARENT);
        d.initStyle(StageStyle.TRANSPARENT);
        d.setScene(sc);
        d.show();

        FadeTransition fadeIn = new FadeTransition(Duration.millis(200), root);
        fadeIn.setFromValue(0.0);
        fadeIn.setToValue(1.0);
        fadeIn.play();
    }

    private HBox createStatusBar() {
        HBox bar = new HBox();
        bar.setAlignment(Pos.CENTER_LEFT);
        bar.setPadding(new Insets(10, 24, 10, 24));
        bar.setStyle("-fx-background-color: #0A0A0E;");

        statusLabel = new Label(t("status_idle"));
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
        if (primaryStage != null) {
            d.initOwner(primaryStage);
            d.initModality(Modality.APPLICATION_MODAL);
        }

        final double[] off = new double[2];

        VBox root = new VBox(15);
        root.setPadding(new Insets(25));
        root.setStyle("-fx-background-color: #0E0E12; -fx-border-color: #1C1C24;");
        root.setOnMousePressed(e -> { off[0] = e.getScreenX() - d.getX(); off[1] = e.getScreenY() - d.getY(); });
        root.setOnMouseDragged(e -> { d.setX(e.getScreenX() - off[0]); d.setY(e.getScreenY() - off[1]); });

        Text title = new Text(t("dialog_add_title"));
        title.setFont(Font.font("Segoe UI", FontWeight.BOLD, 16));
        title.setFill(TEXT);

        Text procLbl = new Text(t("dialog_process_name"));
        procLbl.setFont(Font.font("Segoe UI", 12));
        procLbl.setFill(TEXT);

        ListView<String> runningList = new ListView<>();
        runningList.setPrefHeight(180);
        runningList.getItems().addAll(getRunningProcesses());
        runningList.setStyle("-fx-background-color: #0E0E12; -fx-border-color: #1C1C24;");
        runningList.setCellFactory(lv -> new ListCell<String>() {
            @Override protected void updateItem(String item, boolean empty) {
                super.updateItem(item, empty);
                setText(empty || item == null ? null : item);
                if (empty || item == null) {
                    setStyle("-fx-background-color: transparent; -fx-text-fill: #DCDCE6; -fx-font-size: 12px; -fx-padding: 4 8;");
                } else if (isSelected()) {
                    setStyle("-fx-background-color: #161620; -fx-text-fill: #DCDCE6; -fx-font-size: 12px; -fx-padding: 4 8;");
                } else {
                    setStyle("-fx-background-color: transparent; -fx-text-fill: #DCDCE6; -fx-font-size: 12px; -fx-padding: 4 8;");
                }
            }
        });

        Text err = new Text("");
        err.setFill(RED);

        Button cancel = accentBtn(t("btn_cancel"));
        cancel.setOnAction(e -> d.close());

        Button add = coloredBtn(t("btn_add"), ACCENT);
        add.setOnAction(e -> {
            String sel = runningList.getSelectionModel().getSelectedItem();
            if (sel == null || sel.isEmpty()) {
                err.setText(t("err_enter_process"));
                return;
            }
            String p = sel.trim();
            if (!p.toLowerCase().endsWith(".exe")) {
                err.setText(t("err_must_exe"));
                return;
            }
            if (globalAppId.isEmpty()) {
                err.setText(t("err_enter_id"));
                return;
            }
            saveConfig(p, globalAppId);
            processList.getItems().clear();
            processList.getItems().addAll(loadConfigs());
            log(t("log_added").replace("{0}", p));
            d.close();
        });

        HBox btns = new HBox(10);
        btns.setAlignment(Pos.CENTER_RIGHT);
        btns.getChildren().addAll(cancel, add);

        root.getChildren().addAll(title, procLbl, runningList, err, btns);

        Scene s = new Scene(root, 420, 360);
        s.setFill(Color.TRANSPARENT);
        d.setScene(s);
        d.centerOnScreen();

        FadeTransition fadeIn = new FadeTransition(Duration.millis(200), root);
        fadeIn.setFromValue(0.0);
        fadeIn.setToValue(1.0);
        fadeIn.play();

        d.show();
    }

    private java.util.List<String> getRunningProcesses() {
        java.util.List<String> processes = new java.util.ArrayList<>();
        try {
            ProcessBuilder pb = new ProcessBuilder("tasklist", "/FO", "CSV", "/NH");
            pb.redirectErrorStream(true);
            Process proc = pb.start();
            try (BufferedReader r = new BufferedReader(new InputStreamReader(proc.getInputStream(), StandardCharsets.UTF_8))) {
                String line;
                while ((line = r.readLine()) != null) {
                    line = line.trim();
                    if (line.isEmpty()) continue;
                    String[] parts = line.split(",");
                    if (parts.length > 0) {
                        String name = parts[0].replace("\"", "").trim();
                        if (name.toLowerCase().endsWith(".exe") && !name.equalsIgnoreCase("svchost.exe")
                            && !name.equalsIgnoreCase("csrss.exe") && !name.equalsIgnoreCase("lsass.exe")
                            && !name.equalsIgnoreCase("services.exe") && !name.equalsIgnoreCase("wininit.exe")
                            && !name.equalsIgnoreCase("winlogon.exe") && !name.equalsIgnoreCase("dwm.exe")
                            && !name.equalsIgnoreCase("explorer.exe") && !name.equalsIgnoreCase("taskhostw.exe")
                            && !name.equalsIgnoreCase("RuntimeBroker.exe") && !name.equalsIgnoreCase("ShellExperienceHost.exe")
                            && !name.equalsIgnoreCase("SearchUI.exe") && !name.equalsIgnoreCase("StartMenuExperienceHost.exe")) {
                            processes.add(name);
                        }
                    }
                }
            }
            proc.waitFor();
        } catch (IOException | InterruptedException e) { }
        java.util.Collections.sort(processes, String.CASE_INSENSITIVE_ORDER);
        return processes;
    }

    private TextField makeField(String prompt) {
        TextField f = new TextField();
        f.setPromptText(prompt);
        f.setFont(Font.font("Segoe UI", 13));
        f.setStyle("-fx-background-color: #141418; -fx-text-fill: #DCDCE6; -fx-prompt-text-fill: #828291; -fx-border-color: #1C1C24; -fx-padding: 10;");
        return f;
    }

    private void startEngine() {
        try {
            String exe = findExe();
            if (exe == null) { log("[XGS] Engine not found"); return; }
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
                        Platform.runLater(() -> {
                            log(l);
                            if (l.contains("[XGS] Starting engine")) {
                                statusLabel.setText(t("status_starting"));
                                statusLabel.setTextFill(Color.rgb(255, 200, 50));
                            } else if (l.contains("[XGS] Loading configs")) {
                                statusLabel.setText(t("status_loading"));
                                statusLabel.setTextFill(Color.rgb(255, 200, 50));
                            } else if (l.contains("[XGS] Scanning for processes")) {
                                statusLabel.setText(t("status_scanning"));
                                statusLabel.setTextFill(Color.rgb(255, 200, 50));
                            } else if (l.contains("[XGS] Connecting to Discord")) {
                                statusLabel.setText(t("status_connecting"));
                                statusLabel.setTextFill(Color.rgb(180, 140, 255));
                            } else if (l.contains("[XGS] Running")) {
                                statusLabel.setText(t("status_running"));
                                statusLabel.setTextFill(GREEN);
                            }
                        });
                    }
                } catch (IOException e) { }
            }, "log-reader").start();

            statusLabel.setText(t("status_starting"));
            statusLabel.setTextFill(Color.rgb(255, 200, 50));
            log(t("log_engine_started"));
        } catch (IOException e) {
            log("[XGS] Error: " + e.getMessage());
            statusLabel.setText(t("status_error"));
            statusLabel.setTextFill(RED);
        }
    }

    private void stopEngine() {
        if (engineProcess != null && engineProcess.isAlive()) {
            engineProcess.destroy();
            statusLabel.setText(t("status_stopped"));
            statusLabel.setTextFill(RED);
            log(t("log_engine_stopped"));
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
        File[] files = dir.listFiles((d, n) -> n.endsWith(".json") && !n.equals("settings.json"));
        if (files == null) return new String[0];
        String[] r = new String[files.length];
        for (int i = 0; i < files.length; i++) r[i] = files[i].getName().replace(".json", "");
        return r;
    }

    private void saveConfig(String process, String appId) {
        try {
            new File(CONFIG_DIR).mkdirs();
            String json = "{\n"
                + "  \"process_name\": \"" + esc(process) + "\",\n"
                + "  \"discord_app_id\": \"" + esc(appId) + "\",\n"
                + "  \"scan_type\": \"offsets\",\n"
                + "  \"requires_elevation\": false,\n"
                + "  \"pointers\": {\n"
                + "    \"value\": {\n"
                + "      \"base\": \"" + esc(process.replace(".exe", "")) + ".exe+0x0\",\n"
                + "      \"offsets\": [0]\n"
                + "    }\n"
                + "  },\n"
                + "  \"rpc_template\": {\n"
                + "    \"details\": \"Playing " + esc(process.replace(".exe", "")) + "\",\n"
                + "    \"state\": \"In Game\",\n"
                + "    \"large_image\": \"game_logo\",\n"
                + "    \"large_image_text\": \"" + esc(process.replace(".exe", "")) + "\",\n"
                + "    \"small_image\": \"discord\",\n"
                + "    \"small_image_text\": \"XGameStats\"\n"
                + "  }\n"
                + "}";
            Files.write(new File(CONFIG_DIR, process.replace(".exe", "").toLowerCase() + ".json").toPath(), json.getBytes(StandardCharsets.UTF_8));
        } catch (IOException e) { log("[XGS] Error: " + e.getMessage()); }
    }

    private void writeFile(java.nio.file.Path path, byte[] data) {
        try { Files.write(path, data); } catch (IOException e) { }
    }

    private void centerOnScreen(Stage s) {
        var b = Screen.getPrimary().getBounds();
        s.setX((b.getWidth() - s.getWidth()) / 2);
        s.setY((b.getHeight() - s.getHeight()) / 2);
    }

    public static void main(String[] args) { launch(args); }
}
