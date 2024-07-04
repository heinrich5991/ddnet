#ifndef PROXY_HACKS_SEND_H
#define PROXY_HACKS_SEND_H
class CNetChunk2;

typedef void (*HACKS_SEND_FUNC)(const CNetChunk2 &Packet, void *pUser);

class CSendFunction
{
	HACKS_SEND_FUNC m_pfnSendFunc;
	void *m_pUser;

public:
	CSendFunction(HACKS_SEND_FUNC pfnSendFunc, void *pUser) :
		m_pfnSendFunc(pfnSendFunc),
		m_pUser(pUser)
	{
	}
	void Send(const CNetChunk2 &Chunk)
	{
		m_pfnSendFunc(Chunk, m_pUser);
	}
};
#endif // PROXY_HACKS_SEND_H
