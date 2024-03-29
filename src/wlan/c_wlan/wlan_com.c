/*
* using ioctl based on linux/wireless.h
*/

#include <linux/wireless.h>
#include <sys/ioctl.h>
#include <sys/types.h>
#include <sys/socket.h>
#include <string.h>
#include <unistd.h>
#include <stdio.h>
#include "wlan_com.h"

#define MONITOR_MODE 6
#define MANAGED_MODE 2
#define MAX_CHANNEL 18


//creates a socket for ioctl 
int create_socket(){
  int sock = socket(AF_INET,SOCK_DGRAM,0);
  return (sock>=0) ? sock : -1;
}

int c_get_channel(char *iface){
  //create socket
  //TODO: make sure that works
  int skfd = create_socket();
  if(skfd < 0) return -1;
  struct iwreq wrq;
  memset(&wrq, 0, sizeof(struct iwreq));
  strncpy(wrq.ifr_name,iface,IFNAMSIZ-1);
  ioctl(skfd,SIOCGIWFREQ,&wrq);
  unsigned int freq = wrq.u.freq.m;
  unsigned short channel = (freq - 2407) / 5;
  return channel;

}
int c_switch_channel(char *iface, unsigned int channel){
  if(channel > MAX_CHANNEL) return -1;
  //create socket
  int skfd = create_socket();
  if(skfd < 0) return -1;
  struct iwreq wrq;
  strncpy(wrq.ifr_name,iface,IFNAMSIZ-1);
  wrq.u.freq.m = (long) channel;
  wrq.u.freq.e = 0;
  wrq.u.freq.flags = IW_FREQ_FIXED;
  return ioctl(skfd,SIOCSIWFREQ,&wrq);

}

int c_toggle_power(char *iface,bool state){
  //create socket
  int skfd = create_socket();
  if(skfd < 0) return -1;
  struct ifreq ifr;
  strncpy(ifr.ifr_name,iface,IFNAMSIZ-1);
  if(state){
    ifr.ifr_flags |= IFF_UP;
  }else{
      ifr.ifr_flags &= ~IFF_UP;
  }
  return ioctl(skfd,SIOCSIFFLAGS,&ifr);
}

int c_toggle_monitor_mode(char *iface,bool state){
  //create socket
  int skfd = socket(AF_INET,SOCK_DGRAM,0);
  if(skfd < 0) return -1;
  //create the request
  struct iwreq  wrq;
  //add the iface to the request
  strncpy(wrq.ifr_name,iface,IFNAMSIZ-1);
  //update the mode
  if(state){
    wrq.u.mode = MONITOR_MODE;
  }else{
    wrq.u.mode = MANAGED_MODE;
  }
  int res = ioctl(skfd, SIOCSIWMODE, &wrq);
  close(skfd);
  return res;
}



