package com.xgs.installer;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.StandardCopyOption;

import javafx.animation.FadeTransition;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.control.CheckBox;
import javafx.scene.control.TextField;
import javafx.scene.layout.BorderPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.Region;
import javafx.scene.layout.VBox;
import javafx.scene.paint.Color;
import javafx.scene.text.Font;
import javafx.scene.text.FontWeight;
import javafx.scene.text.Text;
import javafx.stage.DirectoryChooser;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import javafx.util.Duration;

public class Installer extends Application {

    private static final String APP_NAME = "XGameStats Installer";
    private static final String VERSION  = "0.7.0";

    private static final Color ACCENT  = Color.rgb(88, 166, 255);
    private static final Color TEXT    = Color.rgb(230, 230, 238);
    private static final Color DIM     = Color.rgb(130, 130, 145);
    private static final Color GREEN   = Color.rgb(66, 185, 130);
    private static final Color RED     = Color.rgb(235, 80, 80);
    private static final Color YELLOW  = Color.rgb(255, 200, 50);

    private double dragOffsetX, dragOffsetY;

    public static void main(String[] args) { launch(args); }

    @Override
    public void start(Stage stage) {
        BorderPane root = new BorderPane();
        root.setStyle("-fx-background-color: #121218;");

        HBox header = createHeader(stage);
        root.setTop(header);
        root.setCenter(createMainView(stage, null));

        Scene s = new Scene(root, 540, 460);
        stage.initStyle(StageStyle.UNDECORATED);
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
            Platform.exit();
            System.exit(0);
        });
    }

    private HBox createHeader(Stage stage) {
        HBox h = new HBox();
        h.setAlignment(Pos.CENTER_LEFT);
        h.setPadding(new Insets(12, 20, 12, 20));
        h.setStyle("-fx-background-color: #161620;");

        VBox tb = new VBox(2);
        Text t = new Text(APP_NAME);
        t.setFont(Font.font("Segoe UI", FontWeight.BOLD, 18));
        t.setFill(ACCENT);
        Text sub = new Text("v" + VERSION);
        sub.setFont(Font.font("Segoe UI", 11));
        sub.setFill(DIM);
        tb.getChildren().addAll(t, sub);

        Region spacer = new Region();
        HBox.setHgrow(spacer, Priority.ALWAYS);

        Button btnClose = windowBtn("\u00d7");
        btnClose.setOnMouseEntered(ev -> btnClose.setTextFill(RED));
        btnClose.setOnMouseExited(ev  -> btnClose.setTextFill(DIM));
        btnClose.setOnAction(e -> {
            Platform.exit();
            System.exit(0);
        });

        HBox btns = new HBox(8);
        btns.setAlignment(Pos.CENTER_RIGHT);
        btns.getChildren().add(btnClose);
        btns.setOnMousePressed(javafx.event.Event::consume);
        btns.setOnMouseDragged(javafx.event.Event::consume);

        h.getChildren().addAll(tb, spacer, btns);
        return h;
    }

    private VBox createMainView(Stage stage, Text statusText) {
        VBox p = new VBox(14);
        p.setPadding(new Insets(20, 28, 20, 28));

        Text title = new Text("XGameStats");
        title.setFont(Font.font("Segoe UI", FontWeight.BOLD, 22));
        title.setFill(TEXT);

        Text desc = new Text("Universal Discord Rich Presence for singleplayer games.\n"
            + "Select an action below to manage your installation.");
        desc.setFont(Font.font("Segoe UI", 12));
        desc.setFill(DIM);
        desc.setWrappingWidth(480);

        Text pathLabel = new Text("Installation path:");
        pathLabel.setFont(Font.font("Segoe UI", 12));
        pathLabel.setFill(TEXT);

        HBox pathBox = new HBox(8);
        pathBox.setAlignment(Pos.CENTER_LEFT);

        TextField pathField = new TextField(getDefaultInstallPath());
        pathField.setFont(Font.font("Segoe UI", 12));
        pathField.setStyle("-fx-background-color: #1A1A24; -fx-text-fill: #E6E6EE; "
            + "-fx-border-color: #2A2A38; -fx-border-radius: 6; -fx-background-radius: 6; -fx-padding: 8;");
        HBox.setHgrow(pathField, Priority.ALWAYS);

        Button browse = accentBtn("Browse...");
        browse.setOnAction(e -> {
            DirectoryChooser dc = new DirectoryChooser();
            dc.setTitle("Select Directory");
            File dir = dc.showDialog(stage);
            if (dir != null) {
                pathField.setText(dir.getAbsolutePath());
            }
        });
        pathBox.getChildren().addAll(pathField, browse);

        Text optionsLabel = new Text("Options:");
        optionsLabel.setFont(Font.font("Segoe UI", FontWeight.BOLD, 12));
        optionsLabel.setFill(TEXT);

        CheckBox desktopCheck = new CheckBox("Create desktop shortcut");
        desktopCheck.setFont(Font.font("Segoe UI", 12));
        desktopCheck.setTextFill(TEXT);
        desktopCheck.setSelected(true);
        desktopCheck.setStyle("-fx-text-fill: #E6E6EE;");

        CheckBox startMenuCheck = new CheckBox("Create Start Menu shortcut");
        startMenuCheck.setFont(Font.font("Segoe UI", 12));
        startMenuCheck.setTextFill(TEXT);
        startMenuCheck.setSelected(true);
        startMenuCheck.setStyle("-fx-text-fill: #E6E6EE;");

        if (statusText == null) {
            statusText = new Text("");
            statusText.setFont(Font.font("Segoe UI", 12));
        }
        final Text status = statusText;

        Region spacer = new Region();
        VBox.setVgrow(spacer, Priority.ALWAYS);

        HBox btns = new HBox(8);
        btns.setAlignment(Pos.CENTER_RIGHT);

        Button installBtn = coloredBtn("Install", ACCENT);
        installBtn.setOnAction(e -> {
            String path = pathField.getText().trim();
            if (path.isEmpty()) {
                status.setFill(RED);
                status.setText("Please select an installation path");
                return;
            }

            File installDir = new File(path);
            if (installDir.exists() && installDir.list() != null && installDir.list().length > 0) {
                status.setFill(RED);
                status.setText("Directory is not empty. Choose empty folder or delete contents.");
                return;
            }

            status.setFill(YELLOW);
            status.setText("Installing...");

            new Thread(() -> {
                try {
                    performInstall(installDir, desktopCheck.isSelected(), startMenuCheck.isSelected());
                    Platform.runLater(() -> {
                        status.setFill(GREEN);
                        status.setText("Installed successfully! Run xgs.exe to start.");
                    });
                } catch (Exception ex) {
                    Platform.runLater(() -> {
                        status.setFill(RED);
                        status.setText("Error: " + ex.getMessage());
                    });
                }
            }).start();
        });

        Button updateBtn = accentBtn("Update");
        updateBtn.setOnAction(e -> {
            String path = pathField.getText().trim();
            File installDir = new File(path);
            if (!installDir.exists() || !new File(installDir, "xgs.exe").exists()) {
                status.setFill(RED);
                status.setText("Installation not found at: " + path);
                return;
            }

            status.setFill(YELLOW);
            status.setText("Updating...");

            new Thread(() -> {
                try {
                    performUpdate(installDir, desktopCheck.isSelected(), startMenuCheck.isSelected());
                    Platform.runLater(() -> {
                        status.setFill(GREEN);
                        status.setText("Updated successfully!");
                    });
                } catch (Exception ex) {
                    Platform.runLater(() -> {
                        status.setFill(RED);
                        status.setText("Error: " + ex.getMessage());
                    });
                }
            }).start();
        });

        Button repairBtn = accentBtn("Repair");
        repairBtn.setOnAction(e -> {
            String path = pathField.getText().trim();
            File installDir = new File(path);
            if (!installDir.exists() || !new File(installDir, "xgs.exe").exists()) {
                status.setFill(RED);
                status.setText("Installation not found at: " + path);
                return;
            }

            status.setFill(YELLOW);
            status.setText("Repairing...");

            new Thread(() -> {
                try {
                    performRepair(installDir);
                    Platform.runLater(() -> {
                        status.setFill(GREEN);
                        status.setText("Repair complete!");
                    });
                } catch (Exception ex) {
                    Platform.runLater(() -> {
                        status.setFill(RED);
                        status.setText("Error: " + ex.getMessage());
                    });
                }
            }).start();
        });

        Button uninstallBtn = coloredBtn("Uninstall", RED);
        uninstallBtn.setOnAction(e -> {
            String path = pathField.getText().trim();
            File installDir = new File(path);
            if (!installDir.exists()) {
                status.setFill(RED);
                status.setText("Installation not found at: " + path);
                return;
            }

            status.setFill(YELLOW);
            status.setText("Uninstalling...");

            new Thread(() -> {
                try {
                    performUninstall(installDir);
                    Platform.runLater(() -> {
                        status.setFill(GREEN);
                        status.setText("Uninstalled successfully!");
                    });
                } catch (Exception ex) {
                    Platform.runLater(() -> {
                        status.setFill(RED);
                        status.setText("Error: " + ex.getMessage());
                    });
                }
            }).start();
        });

        btns.getChildren().addAll(installBtn, updateBtn, repairBtn, uninstallBtn);

        p.getChildren().addAll(title, desc, pathLabel, pathBox, optionsLabel, desktopCheck, startMenuCheck, status, spacer, btns);
        return p;
    }

    private void performInstall(File targetDir, boolean desktopShortcut, boolean startMenuShortcut) throws IOException {
        File sourceDir = findSourceDir();
        if (sourceDir == null) {
            throw new IOException("Source files not found.");
        }

        targetDir.mkdirs();

        copyFile(new File(sourceDir, "xgs.exe"), new File(targetDir, "xgs.exe"));
        copyFile(new File(sourceDir, "xgamestats-gui.jar"), new File(targetDir, "xgamestats-gui.jar"));
        copyFile(new File(sourceDir, "libscanner.dll"), new File(targetDir, "libscanner.dll"));
        copyFile(new File(sourceDir, "libhooks.dll"), new File(targetDir, "libhooks.dll"));
        copyFile(new File(sourceDir, "translations.json"), new File(targetDir, "translations.json"));
        copyFile(new File(sourceDir, "version.json"), new File(targetDir, "version.json"));
        copyFile(new File(sourceDir, "README.md"), new File(targetDir, "README.md"));
        copyFile(new File(sourceDir, "SECURITY.md"), new File(targetDir, "SECURITY.md"));

        copyDir(new File(sourceDir, "javafx-sdk"), new File(targetDir, "javafx-sdk"));
        copyDir(new File(sourceDir, "configs"), new File(targetDir, "configs"));
        copyDir(new File(sourceDir, "assets"), new File(targetDir, "assets"));

        if (desktopShortcut) {
            createDesktopShortcut(targetDir);
        }
        if (startMenuShortcut) {
            createStartMenuShortcut(targetDir);
        }
    }

    private void performUpdate(File targetDir, boolean desktopShortcut, boolean startMenuShortcut) throws IOException {
        killRunningProcess();
        performInstall(targetDir, desktopShortcut, startMenuShortcut);
    }

    private void performRepair(File installDir) throws IOException {
        File sourceDir = findSourceDir();
        if (sourceDir == null) {
            throw new IOException("Source files not found.");
        }

        String[] required = {"xgs.exe", "xgamestats-gui.jar", "libscanner.dll", "libhooks.dll", "translations.json"};
        for (String file : required) {
            File dst = new File(installDir, file);
            if (!dst.exists()) {
                File src = new File(sourceDir, file);
                if (src.exists()) {
                    copyFile(src, dst);
                }
            }
        }

        copyDir(new File(sourceDir, "javafx-sdk"), new File(installDir, "javafx-sdk"));
        copyDir(new File(sourceDir, "configs"), new File(installDir, "configs"));
        copyDir(new File(sourceDir, "assets"), new File(installDir, "assets"));
    }

    private void performUninstall(File installDir) throws IOException {
        killRunningProcess();

        removeDesktopShortcut();
        removeStartMenuShortcut();

        File configDir = new File(System.getenv("LOCALAPPDATA"), "XGameStats");
        if (configDir.exists()) {
            deleteDir(configDir);
        }

        File[] files = installDir.listFiles();
        if (files != null) {
            for (File f : files) {
                if (f.isDirectory()) {
                    deleteDir(f);
                } else {
                    f.delete();
                }
            }
        }
    }

    private void createDesktopShortcut(File installDir) {
        try {
            String desktop = System.getenv("USERPROFILE") + "\\Desktop";
            String linkPath = desktop + "\\XGameStats.lnk";
            String target = installDir.getAbsolutePath() + "\\xgs.exe";

            String ps = "$s=(New-Object -COM WScript.Shell);"
                + "$lnk=$s.CreateShortcut('" + linkPath + "');"
                + "$lnk.TargetPath='" + target + "';"
                + "$lnk.WorkingDirectory='" + installDir.getAbsolutePath() + "';"
                + "$lnk.Description='XGameStats';"
                + "$lnk.Save();";

            ProcessBuilder pb = new ProcessBuilder("powershell", "-Command", ps);
            pb.redirectErrorStream(true);
            Process proc = pb.start();
            proc.waitFor();
        } catch (Exception e) { }
    }

    private void createStartMenuShortcut(File installDir) {
        try {
            String appData = System.getenv("APPDATA");
            String smPath = appData + "\\Microsoft\\Windows\\Start Menu\\Programs\\XGameStats";
            new File(smPath).mkdirs();

            String linkPath = smPath + "\\XGameStats.lnk";
            String target = installDir.getAbsolutePath() + "\\xgs.exe";

            String ps = "$s=(New-Object -COM WScript.Shell);"
                + "$lnk=$s.CreateShortcut('" + linkPath + "');"
                + "$lnk.TargetPath='" + target + "';"
                + "$lnk.WorkingDirectory='" + installDir.getAbsolutePath() + "';"
                + "$lnk.Description='XGameStats';"
                + "$lnk.Save();";

            ProcessBuilder pb = new ProcessBuilder("powershell", "-Command", ps);
            pb.redirectErrorStream(true);
            Process proc = pb.start();
            proc.waitFor();
        } catch (Exception e) { }
    }

    private void removeDesktopShortcut() {
        try {
            File shortcut = new File(System.getenv("USERPROFILE") + "\\Desktop\\XGameStats.lnk");
            if (shortcut.exists()) {
                shortcut.delete();
            }
        } catch (Exception e) { }
    }

    private void removeStartMenuShortcut() {
        try {
            String appData = System.getenv("APPDATA");
            File smDir = new File(appData + "\\Microsoft\\Windows\\Start Menu\\Programs\\XGameStats");
            if (smDir.exists()) {
                deleteDir(smDir);
            }
        } catch (Exception e) { }
    }

    private File findSourceDir() {
        String[] paths = {"..\\..", "..", "."};
        for (String p : paths) {
            File dir = new File(p);
            if (new File(dir, "xgs.exe").exists() && new File(dir, "xgamestats-gui.jar").exists()) {
                try { return dir.getCanonicalFile(); } catch (IOException e) { }
            }
        }
        return null;
    }

    private void copyFile(File src, File dst) throws IOException {
        if (src.exists()) {
            Files.copy(src.toPath(), dst.toPath(), StandardCopyOption.REPLACE_EXISTING);
        }
    }

    private void copyDir(File src, File dst) throws IOException {
        if (!src.exists()) return;
        dst.mkdirs();
        File[] files = src.listFiles();
        if (files == null) return;
        for (File f : files) {
            File target = new File(dst, f.getName());
            if (f.isDirectory()) {
                copyDir(f, target);
            } else {
                copyFile(f, target);
            }
        }
    }

    private void deleteDir(File dir) {
        if (!dir.exists()) return;
        File[] files = dir.listFiles();
        if (files != null) {
            for (File f : files) {
                if (f.isDirectory()) {
                    deleteDir(f);
                } else {
                    f.delete();
                }
            }
        }
        dir.delete();
    }

    private void killRunningProcess() {
        try {
            ProcessBuilder pb = new ProcessBuilder("taskkill", "/F", "/IM", "xgs.exe");
            pb.redirectErrorStream(true);
            Process proc = pb.start();
            proc.waitFor();
            Thread.sleep(2000);
        } catch (Exception e) { }
    }

    private String getDefaultInstallPath() {
        String pf = System.getenv("ProgramFiles");
        if (pf == null) pf = "C:\\Program Files";
        return pf + "\\XGameStats";
    }

    private Button windowBtn(String symbol) {
        Button b = new Button(symbol);
        b.setFont(Font.font("Segoe UI", FontWeight.BOLD, 14));
        b.setTextFill(DIM);
        b.setStyle("-fx-background-color: transparent; -fx-padding: 4 8; -fx-cursor: hand;");
        b.setMinSize(28, 28);
        b.setMaxSize(28, 28);
        b.setFocusTraversable(false);
        b.setOnMouseEntered(e -> b.setTextFill(TEXT));
        b.setOnMouseExited(e  -> b.setTextFill(DIM));
        return b;
    }

    private Button coloredBtn(String text, Color color) {
        Button b = new Button(text);
        b.setFont(Font.font("Segoe UI", FontWeight.BOLD, 12));
        b.setTextFill(Color.WHITE);
        int r = (int)(color.getRed() * 255);
        int g = (int)(color.getGreen() * 255);
        int bl = (int)(color.getBlue() * 255);
        b.setStyle("-fx-background-color: rgb(" + r + "," + g + "," + bl + "); "
            + "-fx-background-radius: 6; -fx-cursor: hand; -fx-padding: 8 16;");
        return b;
    }

    private Button accentBtn(String text) {
        Button b = new Button(text);
        b.setFont(Font.font("Segoe UI", 12));
        b.setTextFill(TEXT);
        b.setStyle("-fx-background-color: #28283A; -fx-background-radius: 6; -fx-cursor: hand; -fx-padding: 8 14;");
        b.setOnMouseEntered(e -> b.setStyle("-fx-background-color: rgba(88,166,255,0.25); -fx-background-radius: 6; -fx-cursor: hand; -fx-padding: 8 14;"));
        b.setOnMouseExited(e -> b.setStyle("-fx-background-color: #28283A; -fx-background-radius: 6; -fx-cursor: hand; -fx-padding: 8 14;"));
        return b;
    }

    private void centerOnScreen(Stage s) {
        var b = javafx.stage.Screen.getPrimary().getBounds();
        s.setX((b.getWidth() - s.getWidth()) / 2);
        s.setY((b.getHeight() - s.getHeight()) / 2);
    }
}
