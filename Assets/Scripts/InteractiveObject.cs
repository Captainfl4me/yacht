using Unity.Netcode;
using Unity.VisualScripting;
using UnityEngine;

public enum InteractiveType
{
    None,
    Grab,
    Pick,
    Click
}

public abstract class InteractiveObject : NetworkBehaviour
{
    protected InteractiveType interactiveType = InteractiveType.None;
    private ulong? _isInteracting = null;

    private NetworkVariable<Vector3> _networkPosition = new();

    protected virtual void Awake()
    {
        if (_networkPosition.Value.magnitude == 0) {
            _networkPosition.Value = transform.position;
        }
    }


    public InteractiveType GetInteractiveType()
    {
        return interactiveType;
    }

    protected ulong? GetIsInteracting()
    {
        return _isInteracting;
    }

    public abstract bool CanInteract();
    protected abstract void OnInteract();
    protected abstract void OnEndInteraction();

    public bool Interact()
    {
        if (!CanInteract() || !_isInteracting.IsUnityNull()) return false;
        
        _isInteracting = NetworkManager.Singleton.LocalClientId;
        OnInteract();
        return true;
    }
    
    public bool EndInteraction()
    {
        if (_isInteracting.IsUnityNull() || !CanInteract()) return false;

        _isInteracting = null;
        OnEndInteraction();
        return true;
    }

    private void Update()
    {
        if (_isInteracting == NetworkManager.Singleton.LocalClientId)
        {
            _networkPosition.Value = transform.position;
        }
        else if (_isInteracting.IsUnityNull() && NetworkManager.Singleton.IsServer)
        {
            _networkPosition.Value = transform.position;
        }
        else
        {
            transform.position = _networkPosition.Value;
        }
    }
}
