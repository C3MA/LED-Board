#include <Arduino.h>
#include <Ethernet.h>
#include <EthernetUdp.h>

#include "image.hpp"
#include "panel.hpp"

// An EthernetUDP instance to let us send and receive packets over UDP
EthernetUDP Udp;

byte mac[] = { 0xBE, 0xB7, 0x5C, 0x30, 0xC3, 0x04 };

Image image;
Panel panel1(23, 27, 25, 0 * PANEL_WIDTH, 0 * PANEL_HEIGHT);       //data, clock, load
Panel panel2(29, 33, 31, 1 * PANEL_WIDTH, 0 * PANEL_HEIGHT);
Panel panel3(35, 39, 37, 2 * PANEL_WIDTH, 0 * PANEL_HEIGHT);
Panel panel4(41, 45, 43, 3 * PANEL_WIDTH, 0 * PANEL_HEIGHT);
Panel panel5(47, 53, 49, 4 * PANEL_WIDTH, 0 * PANEL_HEIGHT);
#define BITS_OF_BYTE      8

#define UDP_IMAGE_PORT  4242

/**
 *   800 Byte describing all 5 panels in bits
 */
/* uint8_t packetBuffer[(PANEL_WIDTH * PANEL_HEIGHT * MAXIMUM_PANELSIZE) / sizeof(uint8_t)];  // buffer to hold incoming packet,*/
uint8_t packetBuffer[(PANEL_WIDTH * PANEL_HEIGHT * MAXIMUM_PANELSIZE) / BITS_OF_BYTE];
/* Reply with error messages*/
char ReplyBuffer[UDP_TX_PACKET_MAX_SIZE];        // a string to send back

bool someOneIsConnected = false;
/*FIXME hande image 
            if(protocol.isComplete())
            {
              Serial.println("complete");
              panel1.send_image(&image);
              panel2.send_image(&image);
              panel3.send_image(&image);
              panel4.send_image(&image);
              panel5.send_image(&image);
            }

            break;
    }
    */

void receiveUDP() {
  int packetSize = Udp.parsePacket();
  if (packetSize) {
    Serial.print("Received packet of size ");
    Serial.println(packetSize);
    Serial.print("From ");
    IPAddress remote = Udp.remoteIP();
    for (int i=0; i < 4; i++) {
      Serial.print(remote[i], DEC);
      if (i < 3) {
        Serial.print(".");
      }
    }
    Serial.print(", port ");
    Serial.println(Udp.remotePort());
    if (packetSize < ((int) sizeof(packetBuffer)) ) {
      // read the packet into packetBufffer
      Udp.read(packetBuffer, UDP_TX_PACKET_MAX_SIZE);
    }

    if (packetSize == sizeof(packetBuffer)) {

    } else {
      sprintf(ReplyBuffer, "Wrong packet size %d", packetSize);
      // send a reply to the IP address and port that sent us the packet we received
      Udp.beginPacket(Udp.remoteIP(), Udp.remotePort());
      Udp.write(ReplyBuffer);
      Udp.endPacket();
    }
  }
}

void setup() {
    Serial.begin(115200);

    panel1.init();
    panel2.init();
    panel3.init();
    panel4.init();
    panel5.init();
    Serial.print(F("Activate all LEDs\r\n"));
    for (int x = 0; x < IMAGE_WIDTH; x++) {
      for (int y = 0; y < IMAGE_HEIGHT; y++) {
        image.set_pixel(x, y, 1);
      }
    }
    panel1.send_image(&image);
    panel2.send_image(&image);
    panel3.send_image(&image);
    panel4.send_image(&image);
    panel5.send_image(&image);
    

    Ethernet.init();
    Ethernet.begin(mac);
  if (Ethernet.begin(mac) == 0) {
    Serial.println(F("Failed to configure Ethernet using DHCP"));
    if (Ethernet.hardwareStatus() == EthernetNoHardware) {
      Serial.println(F("Ethernet shield was not found.  Sorry, can't run without hardware. :("));
    } else if (Ethernet.linkStatus() == LinkOFF) {
      Serial.println(F("Ethernet cable is not connected."));
    }
    // no point in carrying on, so do nothing forevermore:
    while (true) {
      delay(1);
    }
  }
  Serial.print(F("My IP address: "));
  Serial.println(Ethernet.localIP());

  // start UDP
  Udp.begin(UDP_IMAGE_PORT);
}



// 0x00 0x00 0x00 0x00 0x00 0x00 0x00...
// Width     Height    Delay     Pixel

void default_image(Image* i) {
  static int offset = 0;

  // maximum size of image defined in constructor

  // toggle all pixels in tilted bars
  int dim = max(IMAGE_WIDTH, IMAGE_HEIGHT);
  for (int n = 0; n < dim; n++) {
    int x = (n + offset) % IMAGE_WIDTH;
    int y = n % IMAGE_HEIGHT;

    byte pixel = i->get_pixel(x, y);
    i->set_pixel(x, y, !pixel);
  }
  offset++;
}

void loop() {
    receiveUDP();
    delay(10);
    if (someOneIsConnected == false) {
        default_image(&image);
        panel1.send_image(&image);
        panel2.send_image(&image);
        panel3.send_image(&image);
        panel4.send_image(&image);
        panel5.send_image(&image);
      Serial.println(F(".\n"));
    }
}