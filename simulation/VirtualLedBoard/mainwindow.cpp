#include "mainwindow.h"
#include "ui_mainwindow.h"

#include <QPainter>
#include <QPixmap>
#include <QGraphicsPixmapItem>
#include <QGraphicsView>

MainWindow::MainWindow(QWidget *parent)
    : QMainWindow(parent)
    , ui(new Ui::MainWindow)
{
    ui->setupUi(this);
    this->mOffscreenPanel = new QImage(DEFAULT_WIDTH + LED_DISTANCE, DEFAULT_HEIGHT + LED_DISTANCE, QImage::Format_RGB32);
    this->drawImage(this->mOffscreenPanel);
    this->server = new UdpLedServer (NULL, this);
}

MainWindow::~MainWindow()
{
    delete ui;
}

void MainWindow::drawImage(QImage *target) {
    (void)target; /* handle unused variable ;-) */
    this->mScene=new QGraphicsScene() ;
    QGraphicsView *graphicsView = new QGraphicsView();
    graphicsView->setRenderHints(QPainter::Antialiasing | QPainter::SmoothPixmapTransform);
    graphicsView->setScene(this->mScene);
    graphicsView->show();
    this->ui->ledPanel->addWidget(graphicsView);
}


void MainWindow::setLED(uint8_t x, uint8_t y, bool state) {
    /* draw inital screen */
    QPainter painter(this->mOffscreenPanel);
    painter.setRenderHint(QPainter::Antialiasing, true);
    if (state) {
        painter.setPen(QPen(COLOR_FOREGROUND, 1));
        painter.setBrush(COLOR_FOREGROUND);
    } else {
        painter.setPen(QPen(COLOR_FOREGROUND_OFF, 1));
        painter.setBrush(COLOR_FOREGROUND_OFF);
    }
    painter.drawEllipse(LED_DIAMETER/2 + (x* LED_DISTANCE), 
                        LED_DIAMETER/2 + (y * LED_DISTANCE), 
                        LED_DIAMETER, LED_DIAMETER);
}

void MainWindow::updatePanel(void) {

    QGraphicsPixmapItem* item = new QGraphicsPixmapItem(QPixmap::fromImage(*(this->mOffscreenPanel)));
    this->mScene->clear();
    mScene->addItem(item);
    this->renderPanel();
}

void MainWindow::renderPanel(void) {
    this->mOffscreenPanel->fill(COLOR_BACKGROUND);
}
