#ifndef ANTIBOT_ANTIBOT_INTERFACE_H
#define ANTIBOT_ANTIBOT_INTERFACE_H

#include "antibot_data.h"
extern "C"
{

void AntibotInit(CAntibotData *pData);
void AntibotUpdateData(void);
void AntibotDestroy(void);
void AntibotDump(void);
void AntibotOnPlayerInit(int ClientID);
void AntibotOnPlayerDestroy(int ClientID);
void AntibotOnSpawn(int ClientID);
void AntibotOnHammerFireReloading(int ClientID);
void AntibotOnHammerFire(int ClientID);
void AntibotOnHammerHit(int ClientID);
void AntibotOnDirectInput(int ClientID);
void AntibotOnTick(int ClientID);
void AntibotOnHookAttach(int ClientID, bool Player);
void AntibotOnEngineClientJoin(int ClientID);
void AntibotOnEngineClientDrop(int ClientID, const char *pReason);
void AntibotOnEngineClientMessage(int ClientID, const void *pData, int Size, int Flags);

}

#endif // ANTIBOT_ANTIBOT_INTERFACE_H
