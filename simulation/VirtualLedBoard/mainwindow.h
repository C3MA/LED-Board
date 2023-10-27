#ifndef MAINWINDOW_H
#define MAINWINDOW_H

#include <QMainWindow>
#include <QImage>
#include <QGraphicsScene>
#include "udpserver.h"

#include "settings.h"

#define COLOR_BACKGROUND    Qt::black
#define COLOR_FOREGROUND    QColor(255, 127, 0, 255)
#define COLOR_FOREGROUND_OFF    QColor(25, 25, 25, 255)

QT_BEGIN_NAMESPACE
namespace Ui { class MainWindow; }
QT_END_NAMESPACE

/* dummy */
class UdpLedServer;

class MainWindow : public QMainWindow
{
    Q_OBJECT

public:
    MainWindow(QWidget *parent = nullptr);
    ~MainWindow();

public slots:
    void setLED(uint8_t x, uint8_t y, bool state);
    void updatePanel(void);

private:
    Ui::MainWindow *ui;
    UdpLedServer *server            = nullptr;
    QImage       *mOffscreenPanel   = nullptr;
    QGraphicsScene* mScene          = nullptr;
    void drawImage(QImage *image);
    void renderPanel(void);
};
#endif // MAINWINDOW_H
