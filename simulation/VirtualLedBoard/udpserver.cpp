#include "udpserver.h"
#include "settings.h"
#include <QUdpSocket>
#include <QNetworkDatagram>
#include <ostream>
#include "mainwindow.h"

#define UDP_IMAGE_PORT  4242

UdpLedServer::UdpLedServer (QObject *parent, MainWindow *window)
    : QObject(parent)
{
    initSocket();
    connect(this,
            &UdpLedServer::changeLEDstate,
            window,
            &MainWindow::setLED);
    connect(this,
            &UdpLedServer::updatePanelContent,
            window,
            &MainWindow::updatePanel);

    bool init = true;
    for(int x=0; x < (PANEL_WIDTH * MAXIMUM_PANELSIZE); x++) {
        for(int y=0; y < PANEL_HEIGHT; y++) {
            changeLEDstate(x, y, init);
        }
    }
    updatePanelContent();

}

void UdpLedServer ::initSocket()
{
    this->mUdpSocket = new QUdpSocket(this);
    this->mUdpSocket->bind(QHostAddress::LocalHost, UDP_IMAGE_PORT);

    connect(this->mUdpSocket, &QUdpSocket::readyRead,
            this, &UdpLedServer ::readPendingDatagrams);
}


void UdpLedServer ::readPendingDatagrams()
{
    while (this->mUdpSocket->hasPendingDatagrams()) {
        QNetworkDatagram datagram = this->mUdpSocket->receiveDatagram();
        processTheDatagram(datagram);
    }
}

void UdpLedServer::processTheDatagram(QNetworkDatagram datagram) {
    if (datagram.isValid() && datagram.data().length() == PACKET_LENGTH) {
        qInfo("Received regular datagram.");
        uint8_t brightness = datagram.data().at(PACKET_INDEX_BRIGHTNESS);
        int currentIndex = PACKET_INDEX_PANEL0;

        uint16_t mask = 1;
        for(int y=0; y < PANEL_HEIGHT; y++) {
            for(int x=0; x < (PANEL_WIDTH * MAXIMUM_PANELSIZE); x++) {
              /*
               * The datagram.data().at(currentIndex) here is a char with packed
               * bits. Each bit needs to be separately extracted and hence the
               * `mask` is used for this. A bit-wise shift is applied to advance
               * to the next bit in the char until the char is exhausted. Then
               * the next char from the datagram is read.
               */
              bool state = mask & datagram.data().at(currentIndex);
              this->changeLEDstate(x, y, state);
              if (state) {
                qDebug() << x << "x" << y << " set";
                }
                mask = (mask << 1);
                if (mask >= 256) {
                    mask = 1;
                    currentIndex++;
                }
            }
        }
        this->updatePanelContent();

        qDebug() << "Received datagram:" << brightness;

    } else if (datagram.isValid() && datagram.data().length() != PACKET_LENGTH) {
        qDebug("Received status-check datagram.");
        //socket = new QUdpSocket(this);
        this->mUdpSocket->writeDatagram(datagram.data(), sizeof(datagram.data()), datagram.senderAddress(), datagram.senderPort());
        //this->mUdpSocket->writeDatagram(datagram);
        qDebug("Returned datagram");
    } else {
        qDebug("Received invalid datagram.");
    }
}
