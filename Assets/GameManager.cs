using Unity.Netcode;
using UnityEngine;
using System.Collections.Generic;

public class GameManager : NetworkBehaviour
{
    [SerializeField] private NetworkManager networkManager;
    [SerializeField] private Vector2 playerPosition = Vector2.zero;
    [SerializeField] private Vector2 deckPosition = Vector2.zero;
    [SerializeField] private GameObject deckPrefab;

    private bool gameStarted = false;

    [SerializeField] private GameObject dicePrefab;
    private List<GameObject> dicesToPlay = new List<GameObject>();
    private List<GameObject> dicesToKeep = new List<GameObject>();

    public static GameManager instance;

    private void Awake()
    {
        GameManager.instance = this;
        networkManager.OnClientConnectedCallback += NetworkManager_OnClientConnected;
    }

    private void NetworkManager_OnClientConnected(ulong clientId)
    {
        if (!networkManager.IsHost) return;

        if (gameStarted)
        {
            Debug.Log("Game already started!");
            networkManager.DisconnectClient(clientId);
            return;
        }

        float split_table_angle = 360 / networkManager.ConnectedClientsList.Count;
        GameObject[] players_list = GameObject.FindGameObjectsWithTag("Player");

        for (int i = 0; i < players_list.Length; i++)
        {
            players_list[i].transform.SetPositionAndRotation(new Vector3(
                Mathf.Cos(split_table_angle * i * Mathf.Deg2Rad) * playerPosition.x,
                playerPosition.y,
                Mathf.Sin(split_table_angle * i * Mathf.Deg2Rad) * playerPosition.x
            ), Quaternion.Euler(0, -90.0f + split_table_angle * i, 0));

            GameObject deck = Instantiate(deckPrefab);
            deck.name = "Deck_" + i;
            deck.transform.SetPositionAndRotation(new Vector3(
                Mathf.Cos(split_table_angle * i * Mathf.Deg2Rad) * deckPosition.x,
                deckPosition.y,
                Mathf.Sin(split_table_angle * i * Mathf.Deg2Rad) * deckPosition.x
            ), Quaternion.Euler(0, -90.0f + split_table_angle * i, 0));
            players_list[i].GetComponent<PlayerManager>().deckTransform = deck.transform;
            deck.GetComponent<NetworkObject>().Spawn(true);
        }
        Debug.Log("New player!");
    }

    public int getNumberOfDicesToPlay()
    {
        return 6 - dicesToKeep.Count;
    }

    public void resetPlayingDice()
    {
        foreach (GameObject dice in dicesToPlay)
        {
            Destroy(dice);
        }
        dicesToPlay.Clear();
    }

    public void spawnDice(Transform position)
    {
        if ((dicesToPlay.Count + dicesToKeep.Count) < 6)
        {
            Debug.Log("Spawning dice");
            dicesToPlay.Add(Instantiate(dicePrefab, position.position, Random.rotation));
        }
        else
        {
            Debug.LogError("No more dices allowed");
        }
    }

	public bool KeepDice(GameObject dice) {
		if (dicesToPlay.Remove(dice)) {
			dicesToKeep.Add(dice);
			return true;
		}
		return false;
	}
	public List<GameObject> GetDicesToKeep() {
		return dicesToKeep;
	}
	public bool PlayDice(GameObject dice) {
		if (dicesToKeep.Remove(dice)) {
			dicesToPlay.Add(dice);
			return true;
		}
		return false;
	}
	public List<GameObject> GetDicesToPlay() {
		return dicesToPlay;
	}

    private void Start()
    {
        if (networkManager.IsHost)
        {
            gameStarted = true;
            NextRound();
        }
    }

    private void NextRound()
    {
        networkManager.StartClient();
    }
}
