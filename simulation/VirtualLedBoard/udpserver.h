#ifndef UDPSERVER_H
#define UDPSERVER_H

#include <QAbstractItemModel>
#include <QUdpSocket>
#include "mainwindow.h"

class MainWindow;

class UdpLedServer : public QObject
{
    Q_OBJECT

public:
    /*UdpLedServer (QObject *parent = nullptr); */
    UdpLedServer (QObject *parent = nullptr, MainWindow *window = nullptr);

private:
    void initSocket();
    void readPendingDatagrams();
    QUdpSocket *mUdpSocket;
    void processTheDatagram(QNetworkDatagram datagram);

signals:
    void changeLEDstate(uint8_t x, uint8_t y, bool state);
    void updatePanelContent(void);
};

#endif // UDPSERVER_H
