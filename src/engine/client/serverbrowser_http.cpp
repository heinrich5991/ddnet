#include "serverbrowser_http.h"

#include "http.h"

#include <engine/engine.h>
#include <engine/external/json-parser/json.h>
#include <engine/serverbrowser.h>
#include <engine/shared/serverinfo.h>

#include <memory>

class CServerBrowserHttp : public IServerBrowserHttp
{
public:
	CServerBrowserHttp(IEngine *pEngine);
	virtual ~CServerBrowserHttp() {}
	void Update();
	bool IsRefreshing() { return (bool)m_pGetServers; }
	void Refresh();

	int NumServers() const
	{
		return m_aServers.size();
	}
	const NETADDR &ServerAddress(int Index) const
	{
		return m_aServers[Index].m_Addr;
	}
	void Server(int Index, NETADDR *pAddr, CServerInfo *pInfo) const
	{
		const CEntry &Entry = m_aServers[Index];
		*pAddr = Entry.m_Addr;
		*pInfo = Entry.m_Info;
	}
	int NumLegacyServers() const
	{
		return m_aLegacyServers.size();
	}
	const NETADDR &LegacyServer(int Index) const
	{
		return m_aLegacyServers[Index];
	}

	bool Parse(json_value *pJson);
private:
	class CEntry
	{
	public:
		NETADDR m_Addr;
		CServerInfo m_Info;
	};
	IEngine *m_pEngine;
	std::shared_ptr<CGet> m_pGetServers;

	std::vector<CEntry> m_aServers;
	std::vector<NETADDR> m_aLegacyServers;
};

CServerBrowserHttp::CServerBrowserHttp(IEngine *pEngine)
	: m_pEngine(pEngine)
{
}
void CServerBrowserHttp::Update()
{
	if(m_pGetServers)
	{
		json_value *pJson = m_pGetServers->ResultJson();
		// TODO: Handle parsing error!
		if(!pJson)
		{
			return;
		}
		if(Parse(pJson))
		{
			// TODO: Handle error!
		}
		m_pGetServers = NULL;
	}
}
void CServerBrowserHttp::Refresh()
{
	m_pEngine->AddJob(m_pGetServers = std::make_shared<CGet>("https://heinrich5991.de/teeworlds/temp/servers.json", CTimeout{0, 0, 0}));
}
bool ServerbrowserParseUrl(NETADDR *pOut, const char *pUrl)
{
	char aHost[128];
	const char *pRest = str_startswith(pUrl, "tw-0.6+udp://");
	if(!pRest)
	{
		return true;
	}
	int Length = str_length(pRest);
	int Start = 0;
	int End = Length;
	for(int i = 0; i < Length; i++)
	{
		if(pRest[i] == '@')
		{
			if(Start != 0)
			{
				// Two at signs.
				return true;
			}
			Start = i + 1;
		}
		else if(pRest[i] == '/' || pRest[i] == '?' || pRest[i] == '#')
		{
			End = i;
			break;
		}
	}
	str_truncate(aHost, sizeof(aHost), pRest + Start, End - Start);
	if(net_addr_from_str(pOut, aHost))
	{
		return true;
	}
	return false;
}
bool CServerBrowserHttp::Parse(json_value *pJson)
{
	std::vector<CEntry> aServers;
	std::vector<NETADDR> aLegacyServers;

	const json_value &Json = *pJson;
	// TODO: Handle errors in a sane way. :(
	const json_value &Servers = Json["servers"];
	const json_value &LegacyServers = Json["servers_legacy"];
	if(Servers.type != json_array
		|| (LegacyServers.type != json_array && LegacyServers.type != json_none))
	{
		return true;
	}
	for(unsigned int i = 0; i < Servers.u.array.length; i++)
	{
		const json_value &Server = Servers[i];
		const json_value &Addresses = Server["addresses"];
		const json_value &Info = Server["info"];
		CServerInfo2 ParsedInfo;
		if(Addresses.type != json_array)
		{
			return true;
		}
		if(CServerInfo2::FromJson(&ParsedInfo, &Info))
		{
			dbg_msg("dbg/serverbrowser", "skipped due to info, i=%d", i);
			// Only skip the current server on parsing
			// failure; the server info is "user input" by
			// the game server and can be set to arbitrary
			// values.
			continue;
		}
		CServerInfo SetInfo = ParsedInfo;
		for(unsigned int a = 0; a < Addresses.u.array.length; a++)
		{
			const json_value &Address = Addresses[a];
			if(Address.type != json_string)
			{
				return true;
			}
			// TODO: Address address handling :P
			NETADDR ParsedAddr;
			if(ServerbrowserParseUrl(&ParsedAddr, Addresses[a]))
			{
				dbg_msg("dbg/serverbrowser", "unknown address, i=%d a=%d", i, a);
				// Skip unknown addresses.
				continue;
			}
			aServers.push_back({ParsedAddr, SetInfo});
		}
	}
	if(LegacyServers.type == json_array)
	{
		for(unsigned int i = 0; i < LegacyServers.u.array.length; i++)
		{
			const json_value &Address = LegacyServers[i];
			NETADDR ParsedAddr;
			if(Address.type != json_string
				|| net_addr_from_str(&ParsedAddr, Address))
			{
				return true;
			}
			aLegacyServers.push_back(ParsedAddr);
		}
	}
	m_aServers = aServers;
	m_aLegacyServers = aLegacyServers;
	return false;
}
IServerBrowserHttp *CreateServerBrowserHttp(IEngine *pEngine)
{
	return new CServerBrowserHttp(pEngine);
}
